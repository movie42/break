export interface GroupPreset {
  id: string;
  name: string;
  hosts: string[];
}

export const GROUP_PRESETS: readonly GroupPreset[] = [
  {
    id: "entertainment",
    name: "엔터테인먼트",
    hosts: [
      "youtube.com",
      "netflix.com",
      "twitch.tv",
      "chzzk.naver.com",
      "sooplive.co.kr",
      "tving.com",
      "wavve.com",
      "coupangplay.com",
      "disneyplus.com",
      "laftel.net",
    ],
  },
  {
    id: "sns",
    name: "SNS",
    hosts: [
      "instagram.com",
      "facebook.com",
      "x.com",
      "twitter.com",
      "threads.net",
      "tiktok.com",
      "reddit.com",
      "band.us",
    ],
  },
  {
    id: "news",
    name: "뉴스",
    hosts: [
      "news.naver.com",
      "news.daum.net",
      "v.daum.net",
      "yna.co.kr",
      "chosun.com",
      "joongang.co.kr",
      "donga.com",
      "hani.co.kr",
      "khan.co.kr",
      "mk.co.kr",
      "hankyung.com",
    ],
  },
  {
    id: "shopping",
    name: "쇼핑",
    hosts: [
      "coupang.com",
      "11st.co.kr",
      "gmarket.co.kr",
      "auction.co.kr",
      "musinsa.com",
      "shopping.naver.com",
      "smartstore.naver.com",
      "kream.co.kr",
      "aliexpress.com",
      "temu.com",
    ],
  },
  {
    id: "games",
    name: "게임",
    hosts: [
      "store.steampowered.com",
      "steamcommunity.com",
      "epicgames.com",
      "roblox.com",
      "nexon.com",
      "op.gg",
      "inven.co.kr",
      "ruliweb.com",
    ],
  },
];
