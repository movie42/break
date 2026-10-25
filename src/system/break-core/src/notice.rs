use chrono::{DateTime, Local, Utc};

use crate::mail::Mail;
use crate::rules::Rules;

fn local(at: DateTime<Utc>) -> String {
    at.with_timezone(&Local).format("%m/%d %H:%M").to_string()
}

pub fn recipients(rules: &Rules) -> Vec<String> {
    rules
        .alerts
        .recipients
        .iter()
        .map(|recipient| recipient.email.clone())
        .collect()
}

pub fn confirmation(email: &str) -> Mail {
    Mail {
        to: vec![email.to_string()],
        subject: "이 알림을 받게 됐습니다".to_string(),
        body: "이 주소가 Break 알림 주소로 등록됐습니다.\n\nBreak는 사이트 차단 앱입니다. 차단 프로그램이 꺼지거나, 차단 중에 앱이 종료되면 이 주소로 알려 드립니다.".to_string(),
    }
}

pub fn removal_planned(rules: &Rules, email: &str, effective: DateTime<Utc>) -> Mail {
    Mail {
        to: recipients(rules),
        subject: "이 주소가 알림에서 빠질 예정입니다".to_string(),
        body: format!(
            "{email} 주소를 Break 알림에서 빼 달라는 신청이 들어왔습니다.\n\n{}에 빠집니다. 그 전에는 앱에서 취소할 수 있습니다.",
            local(effective)
        ),
    }
}

pub fn daemon_stopped(rules: &Rules, at: DateTime<Utc>) -> Mail {
    Mail {
        to: recipients(rules),
        subject: "Break 차단 프로그램이 꺼졌습니다".to_string(),
        body: format!(
            "{}에 Break 차단 프로그램이 멈췄습니다. 다시 켜질 때까지 사이트가 막히지 않습니다.",
            local(at)
        ),
    }
}

pub fn daemon_was_down(rules: &Rules, from: DateTime<Utc>, to: DateTime<Utc>) -> Mail {
    Mail {
        to: recipients(rules),
        subject: "Break 차단 프로그램이 꺼져 있었습니다".to_string(),
        body: format!(
            "{}부터 {}까지 Break 차단 프로그램이 꺼져 있었습니다. 지금은 다시 켜졌습니다.",
            local(from),
            local(to)
        ),
    }
}

pub fn app_quit(rules: &Rules, at: DateTime<Utc>) -> Mail {
    Mail {
        to: recipients(rules),
        subject: "차단 중에 Break 앱이 종료됐습니다".to_string(),
        body: format!(
            "{}에 차단이 걸려 있는 동안 Break 앱이 종료됐습니다. Mac을 끈 경우에도 이 메일이 갈 수 있습니다.",
            local(at)
        ),
    }
}
