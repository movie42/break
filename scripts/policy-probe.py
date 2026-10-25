#!/usr/bin/env python3
import argparse
import ctypes
import ctypes.util
import json
import os
import plistlib
import subprocess
import sys

MANAGED_PREFERENCES_DIR = "/Library/Managed Preferences"
APPLICATIONS_DIR = "/Applications"
KCF_STRING_ENCODING_UTF8 = 0x08000100
KCF_PROPERTY_LIST_XML_FORMAT_V1_0 = 100

cf = ctypes.cdll.LoadLibrary(ctypes.util.find_library("CoreFoundation"))

cf.CFStringCreateWithCString.restype = ctypes.c_void_p
cf.CFStringCreateWithCString.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_uint32]
cf.CFPreferencesCopyAppValue.restype = ctypes.c_void_p
cf.CFPreferencesCopyAppValue.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
cf.CFPreferencesAppValueIsForced.restype = ctypes.c_ubyte
cf.CFPreferencesAppValueIsForced.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
cf.CFPropertyListCreateData.restype = ctypes.c_void_p
cf.CFPropertyListCreateData.argtypes = [
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.c_long,
    ctypes.c_ulong,
    ctypes.c_void_p,
]
cf.CFDataGetBytePtr.restype = ctypes.POINTER(ctypes.c_ubyte)
cf.CFDataGetBytePtr.argtypes = [ctypes.c_void_p]
cf.CFDataGetLength.restype = ctypes.c_long
cf.CFDataGetLength.argtypes = [ctypes.c_void_p]
cf.CFRelease.argtypes = [ctypes.c_void_p]


def cfstr(value):
    return cf.CFStringCreateWithCString(None, value.encode("utf-8"), KCF_STRING_ENCODING_UTF8)


def to_python(ref):
    data = cf.CFPropertyListCreateData(
        None, ref, KCF_PROPERTY_LIST_XML_FORMAT_V1_0, 0, None
    )
    if not data:
        return "<읽을 수 없는 값>"
    try:
        length = cf.CFDataGetLength(data)
        raw = ctypes.string_at(cf.CFDataGetBytePtr(data), length)
        return plistlib.loads(raw)
    finally:
        cf.CFRelease(data)


def probe(bundle_id, key):
    key_ref = cfstr(key)
    app_ref = cfstr(bundle_id)
    try:
        forced = bool(cf.CFPreferencesAppValueIsForced(key_ref, app_ref))
        value_ref = cf.CFPreferencesCopyAppValue(key_ref, app_ref)
        if not value_ref:
            return forced, None
        try:
            return forced, to_python(value_ref)
        finally:
            cf.CFRelease(value_ref)
    finally:
        cf.CFRelease(key_ref)
        cf.CFRelease(app_ref)


def plutil_extract(path, key, fmt):
    result = subprocess.run(
        ["/usr/bin/plutil", "-extract", key, fmt, "-o", "-", path],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def is_chromium(bundle):
    frameworks = os.path.join(bundle, "Contents", "Frameworks")
    if not os.path.isdir(frameworks):
        return False
    for name in os.listdir(frameworks):
        if not name.endswith(".framework"):
            continue
        helpers = os.path.join(frameworks, name, "Versions", "Current", "Helpers")
        if os.path.isdir(helpers):
            return True
    return False


def handles_web(bundle):
    info = os.path.join(bundle, "Contents", "Info.plist")
    raw = plutil_extract(info, "CFBundleURLTypes", "json")
    if not raw:
        return False
    try:
        types = json.loads(raw)
    except json.JSONDecodeError:
        return False
    if not isinstance(types, list):
        return False
    for entry in types:
        schemes = entry.get("CFBundleURLSchemes") if isinstance(entry, dict) else None
        if not isinstance(schemes, list):
            continue
        for scheme in schemes:
            if isinstance(scheme, str) and scheme.lower() in ("http", "https"):
                return True
    return False


def discover():
    ids = set()
    if os.path.isdir(MANAGED_PREFERENCES_DIR):
        for name in os.listdir(MANAGED_PREFERENCES_DIR):
            if name.endswith(".plist"):
                ids.add(name[: -len(".plist")])
    if os.path.isdir(APPLICATIONS_DIR):
        for name in os.listdir(APPLICATIONS_DIR):
            bundle = os.path.join(APPLICATIONS_DIR, name)
            if not name.endswith(".app") or not is_chromium(bundle):
                continue
            if not handles_web(bundle):
                continue
            info = os.path.join(bundle, "Contents", "Info.plist")
            bundle_id = plutil_extract(info, "CFBundleIdentifier", "raw")
            if bundle_id:
                ids.add(bundle_id)
    return sorted(ids)


def main():
    parser = argparse.ArgumentParser(
        description="브라우저가 실제로 읽는 관리 정책 값을 CFPreferences로 확인합니다. "
        "파일 유무가 아니라 cfprefsd가 들고 있는 값을 봅니다."
    )
    parser.add_argument("bundle_ids", nargs="*", help="확인할 번들 ID. 비우면 자동으로 찾습니다")
    parser.add_argument("--key", default="URLBlocklist", help="확인할 정책 키 (기본: URLBlocklist)")
    parser.add_argument(
        "--expect-clear",
        action="store_true",
        help="하나라도 정책이 살아 있으면 종료 코드 1",
    )
    args = parser.parse_args()

    bundle_ids = args.bundle_ids or discover()
    if not bundle_ids:
        print("확인할 번들 ID를 찾지 못했습니다", file=sys.stderr)
        return 2

    forced_any = False
    for bundle_id in bundle_ids:
        forced, value = probe(bundle_id, args.key)
        forced_any = forced_any or forced
        path = os.path.join(MANAGED_PREFERENCES_DIR, f"{bundle_id}.plist")
        exists = "있음" if os.path.exists(path) else "없음"
        print(f"{bundle_id}: forced={forced} value={value!r} file={exists}")

    if args.expect_clear and forced_any:
        print("정책이 아직 살아 있습니다", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
