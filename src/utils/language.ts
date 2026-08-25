// ISO-639-1 + Country Code (Option)
// https://zh.wikipedia.org/wiki/ISO_639-1%E4%BB%A3%E7%A0%81%E8%A1%A8
export const languageList = ['zh_cn', 'en'] as const;

// https://flagicons.lipis.dev/
export enum LanguageFlag {
    zh_cn = 'cn',
    zh_tw = 'cn',
    mn_mo = 'cn',
    en = 'gb',
    ja = 'jp',
    ko = 'kr',
    fr = 'fr',
    es = 'es',
    ru = 'ru',
    de = 'de',
    it = 'it',
    tr = 'tr',
    pt_pt = 'pt',
    pt_br = 'br',
    vi = 'vn',
    id = 'id',
    th = 'th',
    ms = 'ms',
    ar = 'ae',
    hi = 'in',
    km = 'kh',
    mn_cy = 'mn',
    nb_no = 'no',
    nn_no = 'no',
    fa = 'ir',
    sv = 'se',
    pl = 'pl',
    nl = 'nl',
    uk = 'ua',
    he = 'il',
}

export interface Language {
    code: LanguageFlag;
    displayName: string;
}

type UiLanguageType = {
    [K in keyof typeof uiLanguageData]: Language;
} & {
    includes(lang: string): boolean;
    find(predicate: (lang: Language) => boolean): Language | undefined;
};

export const uiLanguageData = {
    zh_cn: { code: LanguageFlag.zh_cn, displayName: '简体中文' },
    en: { code: LanguageFlag.en, displayName: 'English' },
} as const;

export const uiLanguage: UiLanguageType = {
    ...uiLanguageData,
    includes(lang: string): boolean {
        return lang in uiLanguageData;
    },
    find(predicate: (lang: Language) => boolean): Language | undefined {
        const entries = Object.entries(uiLanguageData);
        const found = entries.find(([_, lang]) => predicate(lang));
        return found ? found[1] : undefined;
    }
};
