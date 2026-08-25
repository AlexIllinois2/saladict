import { initReactI18next } from 'react-i18next';
import i18n from 'i18next';
import zh_CN from './locales/zh_CN.json';
import en_US from './locales/en_US.json';

i18n.use(initReactI18next).init({
    fallbackLng: {
        default: ['en'],
    },
    debug: false,
    interpolation: {
        escapeValue: false,
    },
    resources: {
        en: en_US,
        zh_cn: zh_CN,
    },
});

export default i18n;
