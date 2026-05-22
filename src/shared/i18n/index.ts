import i18next, { type Resource } from "i18next";
import {
  APP_LOCALES,
  DEFAULT_ACTIVE_LOCALE,
  FALLBACK_LOCALE,
  SOURCE_LOCALE,
} from "./config";
import type { AppLocale } from "./types";
import commonEn from "./locales/en/common";
import mcpEn from "./locales/en/mcp";
import pluginsEn from "./locales/en/plugins";
import usageEn from "./locales/en/usage";
import contextFilesEn from "./locales/en/contextFiles";
import securityEn from "./locales/en/security";
import curatorEn from "./locales/en/curator";
import navigationEn from "./locales/en/navigation";
import welcomeEn from "./locales/en/welcome";
import setupEn from "./locales/en/setup";
import chatEn from "./locales/en/chat";
import settingsEn from "./locales/en/settings";
import toolsEn from "./locales/en/tools";
import sessionsEn from "./locales/en/sessions";
import modelsEn from "./locales/en/models";
import providersEn from "./locales/en/providers";
import officeEn from "./locales/en/office";
import errorsEn from "./locales/en/errors";
import schedulesEn from "./locales/en/schedules";
import skillsEn from "./locales/en/skills";
import gatewayEn from "./locales/en/gateway";
import agentsEn from "./locales/en/agents";
import soulEn from "./locales/en/soul";
import memoryEn from "./locales/en/memory";
import installEn from "./locales/en/install";
import constantsEn from "./locales/en/constants";
import kanbanEn from "./locales/en/kanban";
import sshEn from "./locales/en/ssh";
import hermesI18nEn from "./locales/en/hermes";
import configI18nEn from "./locales/en/config";
import claw3dI18nEn from "./locales/en/claw3d";
import commonEs from "./locales/es/common";
import navigationEs from "./locales/es/navigation";
import welcomeEs from "./locales/es/welcome";
import setupEs from "./locales/es/setup";
import chatEs from "./locales/es/chat";
import settingsEs from "./locales/es/settings";
import toolsEs from "./locales/es/tools";
import sessionsEs from "./locales/es/sessions";
import modelsEs from "./locales/es/models";
import providersEs from "./locales/es/providers";
import officeEs from "./locales/es/office";
import errorsEs from "./locales/es/errors";
import schedulesEs from "./locales/es/schedules";
import skillsEs from "./locales/es/skills";
import gatewayEs from "./locales/es/gateway";
import agentsEs from "./locales/es/agents";
import soulEs from "./locales/es/soul";
import memoryEs from "./locales/es/memory";
import installEs from "./locales/es/install";
import constantsEs from "./locales/es/constants";
import kanbanEs from "./locales/es/kanban";
import sshEs from "./locales/es/ssh";
import hermesI18nEs from "./locales/es/hermes";
import configI18nEs from "./locales/es/config";
import claw3dI18nEs from "./locales/es/claw3d";
import commonId from "./locales/id/common";
import navigationId from "./locales/id/navigation";
import welcomeId from "./locales/id/welcome";
import setupId from "./locales/id/setup";
import chatId from "./locales/id/chat";
import settingsId from "./locales/id/settings";
import toolsId from "./locales/id/tools";
import sessionsId from "./locales/id/sessions";
import modelsId from "./locales/id/models";
import providersId from "./locales/id/providers";
import officeId from "./locales/id/office";
import errorsId from "./locales/id/errors";
import schedulesId from "./locales/id/schedules";
import skillsId from "./locales/id/skills";
import gatewayId from "./locales/id/gateway";
import agentsId from "./locales/id/agents";
import soulId from "./locales/id/soul";
import memoryId from "./locales/id/memory";
import installId from "./locales/id/install";
import constantsId from "./locales/id/constants";
import kanbanId from "./locales/id/kanban";
import sshId from "./locales/id/ssh";
import hermesI18nId from "./locales/id/hermes";
import configI18nId from "./locales/id/config";
import claw3dI18nId from "./locales/id/claw3d";
import commonZh from "./locales/zh-CN/common";
import navigationZh from "./locales/zh-CN/navigation";
import welcomeZh from "./locales/zh-CN/welcome";
import setupZh from "./locales/zh-CN/setup";
import chatZh from "./locales/zh-CN/chat";
import settingsZh from "./locales/zh-CN/settings";
import toolsZh from "./locales/zh-CN/tools";
import sessionsZh from "./locales/zh-CN/sessions";
import modelsZh from "./locales/zh-CN/models";
import providersZh from "./locales/zh-CN/providers";
import officeZh from "./locales/zh-CN/office";
import errorsZh from "./locales/zh-CN/errors";
import schedulesZh from "./locales/zh-CN/schedules";
import skillsZh from "./locales/zh-CN/skills";
import gatewayZh from "./locales/zh-CN/gateway";
import agentsZh from "./locales/zh-CN/agents";
import soulZh from "./locales/zh-CN/soul";
import memoryZh from "./locales/zh-CN/memory";
import installZh from "./locales/zh-CN/install";
import constantsZh from "./locales/zh-CN/constants";
import kanbanZh from "./locales/zh-CN/kanban";
import sshZh from "./locales/zh-CN/ssh";
import hermesI18nZh from "./locales/zh-CN/hermes";
import configI18nZh from "./locales/zh-CN/config";
import claw3dI18nZh from "./locales/zh-CN/claw3d";
import commonZhTw from "./locales/zh-TW/common";
import navigationZhTw from "./locales/zh-TW/navigation";
import welcomeZhTw from "./locales/zh-TW/welcome";
import setupZhTw from "./locales/zh-TW/setup";
import chatZhTw from "./locales/zh-TW/chat";
import settingsZhTw from "./locales/zh-TW/settings";
import toolsZhTw from "./locales/zh-TW/tools";
import sessionsZhTw from "./locales/zh-TW/sessions";
import modelsZhTw from "./locales/zh-TW/models";
import providersZhTw from "./locales/zh-TW/providers";
import officeZhTw from "./locales/zh-TW/office";
import errorsZhTw from "./locales/zh-TW/errors";
import schedulesZhTw from "./locales/zh-TW/schedules";
import skillsZhTw from "./locales/zh-TW/skills";
import gatewayZhTw from "./locales/zh-TW/gateway";
import agentsZhTw from "./locales/zh-TW/agents";
import soulZhTw from "./locales/zh-TW/soul";
import memoryZhTw from "./locales/zh-TW/memory";
import installZhTw from "./locales/zh-TW/install";
import constantsZhTw from "./locales/zh-TW/constants";
import kanbanZhTw from "./locales/zh-TW/kanban";
import sshZhTw from "./locales/zh-TW/ssh";
import hermesI18nZhTw from "./locales/zh-TW/hermes";
import configI18nZhTw from "./locales/zh-TW/config";
import claw3dI18nZhTw from "./locales/zh-TW/claw3d";
import commonJa from "./locales/ja/common";
import navigationJa from "./locales/ja/navigation";
import welcomeJa from "./locales/ja/welcome";
import setupJa from "./locales/ja/setup";
import chatJa from "./locales/ja/chat";
import settingsJa from "./locales/ja/settings";
import toolsJa from "./locales/ja/tools";
import sessionsJa from "./locales/ja/sessions";
import modelsJa from "./locales/ja/models";
import providersJa from "./locales/ja/providers";
import officeJa from "./locales/ja/office";
import errorsJa from "./locales/ja/errors";
import schedulesJa from "./locales/ja/schedules";
import skillsJa from "./locales/ja/skills";
import gatewayJa from "./locales/ja/gateway";
import agentsJa from "./locales/ja/agents";
import soulJa from "./locales/ja/soul";
import memoryJa from "./locales/ja/memory";
import installJa from "./locales/ja/install";
import constantsJa from "./locales/ja/constants";
import kanbanJa from "./locales/ja/kanban";
import sshJa from "./locales/ja/ssh";
import hermesI18nJa from "./locales/ja/hermes";
import configI18nJa from "./locales/ja/config";
import claw3dI18nJa from "./locales/ja/claw3d";
import commonPt from "./locales/pt-BR/common";
import navigationPt from "./locales/pt-BR/navigation";
import welcomePt from "./locales/pt-BR/welcome";
import setupPt from "./locales/pt-BR/setup";
import chatPt from "./locales/pt-BR/chat";
import settingsPt from "./locales/pt-BR/settings";
import toolsPt from "./locales/pt-BR/tools";
import sessionsPt from "./locales/pt-BR/sessions";
import modelsPt from "./locales/pt-BR/models";
import providersPt from "./locales/pt-BR/providers";
import officePt from "./locales/pt-BR/office";
import errorsPt from "./locales/pt-BR/errors";
import schedulesPt from "./locales/pt-BR/schedules";
import skillsPt from "./locales/pt-BR/skills";
import gatewayPt from "./locales/pt-BR/gateway";
import agentsPt from "./locales/pt-BR/agents";
import soulPt from "./locales/pt-BR/soul";
import memoryPt from "./locales/pt-BR/memory";
import installPt from "./locales/pt-BR/install";
import constantsPt from "./locales/pt-BR/constants";
import kanbanPt from "./locales/pt-BR/kanban";
import sshPt from "./locales/pt-BR/ssh";
import hermesI18nPt from "./locales/pt-BR/hermes";
import configI18nPt from "./locales/pt-BR/config";
import claw3dI18nPt from "./locales/pt-BR/claw3d";
import commonPtPt from "./locales/pt-PT/common";
import navigationPtPt from "./locales/pt-PT/navigation";
import welcomePtPt from "./locales/pt-PT/welcome";
import setupPtPt from "./locales/pt-PT/setup";
import chatPtPt from "./locales/pt-PT/chat";
import settingsPtPt from "./locales/pt-PT/settings";
import toolsPtPt from "./locales/pt-PT/tools";
import sessionsPtPt from "./locales/pt-PT/sessions";
import modelsPtPt from "./locales/pt-PT/models";
import providersPtPt from "./locales/pt-PT/providers";
import officePtPt from "./locales/pt-PT/office";
import errorsPtPt from "./locales/pt-PT/errors";
import schedulesPtPt from "./locales/pt-PT/schedules";
import skillsPtPt from "./locales/pt-PT/skills";
import gatewayPtPt from "./locales/pt-PT/gateway";
import agentsPtPt from "./locales/pt-PT/agents";
import soulPtPt from "./locales/pt-PT/soul";
import memoryPtPt from "./locales/pt-PT/memory";
import installPtPt from "./locales/pt-PT/install";
import constantsPtPt from "./locales/pt-PT/constants";
import kanbanPtPt from "./locales/pt-PT/kanban";
import sshPtPt from "./locales/pt-PT/ssh";
import hermesI18nPtPt from "./locales/pt-PT/hermes";
import configI18nPtPt from "./locales/pt-PT/config";
import claw3dI18nPtPt from "./locales/pt-PT/claw3d";

export const resources = {
  en: {
    translation: {
      common: commonEn,
      plugins: pluginsEn,
      contextFiles: contextFilesEn,
      curator: curatorEn,
      security: securityEn,
      usage: usageEn,
      mcp: mcpEn,
      navigation: navigationEn,
      welcome: welcomeEn,
      setup: setupEn,
      chat: chatEn,
      settings: settingsEn,
      tools: toolsEn,
      sessions: sessionsEn,
      models: modelsEn,
      providers: providersEn,
      office: officeEn,
      errors: errorsEn,
      schedules: schedulesEn,
      skills: skillsEn,
      gateway: gatewayEn,
      agents: agentsEn,
      soul: soulEn,
      memory: memoryEn,
      install: installEn,
      constants: constantsEn,
      kanban: kanbanEn,
      ssh: sshEn,
      hermes: hermesI18nEn,
      config: configI18nEn,
      claw3d: claw3dI18nEn,
    },
  },
  es: {
    translation: {
      common: commonEs,






      navigation: navigationEs,
      welcome: welcomeEs,
      setup: setupEs,
      chat: chatEs,
      settings: settingsEs,
      tools: toolsEs,
      sessions: sessionsEs,
      models: modelsEs,
      providers: providersEs,
      office: officeEs,
      errors: errorsEs,
      schedules: schedulesEs,
      skills: skillsEs,
      gateway: gatewayEs,
      agents: agentsEs,
      soul: soulEs,
      memory: memoryEs,
      install: installEs,
      constants: constantsEs,
      kanban: kanbanEs,
      ssh: sshEs,
      hermes: hermesI18nEs,
      config: configI18nEs,
      claw3d: claw3dI18nEs,
    },
  },
  id: {
    translation: {
      common: commonId,






      navigation: navigationId,
      welcome: welcomeId,
      setup: setupId,
      chat: chatId,
      settings: settingsId,
      tools: toolsId,
      sessions: sessionsId,
      models: modelsId,
      providers: providersId,
      office: officeId,
      errors: errorsId,
      schedules: schedulesId,
      skills: skillsId,
      gateway: gatewayId,
      agents: agentsId,
      soul: soulId,
      memory: memoryId,
      install: installId,
      constants: constantsId,
      kanban: kanbanId,
      ssh: sshId,
      hermes: hermesI18nId,
      config: configI18nId,
      claw3d: claw3dI18nId,
    },
  },
  "zh-CN": {
    translation: {
      common: commonZh,






      navigation: navigationZh,
      welcome: welcomeZh,
      setup: setupZh,
      chat: chatZh,
      settings: settingsZh,
      tools: toolsZh,
      sessions: sessionsZh,
      models: modelsZh,
      providers: providersZh,
      office: officeZh,
      errors: errorsZh,
      schedules: schedulesZh,
      skills: skillsZh,
      gateway: gatewayZh,
      agents: agentsZh,
      soul: soulZh,
      memory: memoryZh,
      install: installZh,
      constants: constantsZh,
      kanban: kanbanZh,
      ssh: sshZh,
      hermes: hermesI18nZh,
      config: configI18nZh,
      claw3d: claw3dI18nZh,
    },
  },
  "zh-TW": {
    translation: {
      common: commonZhTw,






      navigation: navigationZhTw,
      welcome: welcomeZhTw,
      setup: setupZhTw,
      chat: chatZhTw,
      settings: settingsZhTw,
      tools: toolsZhTw,
      sessions: sessionsZhTw,
      models: modelsZhTw,
      providers: providersZhTw,
      office: officeZhTw,
      errors: errorsZhTw,
      schedules: schedulesZhTw,
      skills: skillsZhTw,
      gateway: gatewayZhTw,
      agents: agentsZhTw,
      soul: soulZhTw,
      memory: memoryZhTw,
      install: installZhTw,
      constants: constantsZhTw,
      kanban: kanbanZhTw,
      ssh: sshZhTw,
      hermes: hermesI18nZhTw,
      config: configI18nZhTw,
      claw3d: claw3dI18nZhTw,
    },
  },
  "pt-BR": {
    translation: {
      common: commonPt,






      navigation: navigationPt,
      welcome: welcomePt,
      setup: setupPt,
      chat: chatPt,
      settings: settingsPt,
      tools: toolsPt,
      sessions: sessionsPt,
      models: modelsPt,
      providers: providersPt,
      office: officePt,
      errors: errorsPt,
      schedules: schedulesPt,
      skills: skillsPt,
      gateway: gatewayPt,
      agents: agentsPt,
      soul: soulPt,
      memory: memoryPt,
      install: installPt,
      constants: constantsPt,
      kanban: kanbanPt,
      ssh: sshPt,
      hermes: hermesI18nPt,
      config: configI18nPt,
      claw3d: claw3dI18nPt,
    },
  },
  "pt-PT": {
    translation: {
      common: commonPtPt,






      navigation: navigationPtPt,
      welcome: welcomePtPt,
      setup: setupPtPt,
      chat: chatPtPt,
      settings: settingsPtPt,
      tools: toolsPtPt,
      sessions: sessionsPtPt,
      models: modelsPtPt,
      providers: providersPtPt,
      office: officePtPt,
      errors: errorsPtPt,
      schedules: schedulesPtPt,
      skills: skillsPtPt,
      gateway: gatewayPtPt,
      agents: agentsPtPt,
      soul: soulPtPt,
      memory: memoryPtPt,
      install: installPtPt,
      constants: constantsPtPt,
      kanban: kanbanPtPt,
      ssh: sshPtPt,
      hermes: hermesI18nPtPt,
      config: configI18nPtPt,
      claw3d: claw3dI18nPtPt,
    },
  },
  ja: {
    translation: {
      common: commonJa,






      navigation: navigationJa,
      welcome: welcomeJa,
      setup: setupJa,
      chat: chatJa,
      settings: settingsJa,
      tools: toolsJa,
      sessions: sessionsJa,
      models: modelsJa,
      providers: providersJa,
      office: officeJa,
      errors: errorsJa,
      schedules: schedulesJa,
      skills: skillsJa,
      gateway: gatewayJa,
      agents: agentsJa,
      soul: soulJa,
      memory: memoryJa,
      install: installJa,
      constants: constantsJa,
      kanban: kanbanJa,
      ssh: sshJa,
      hermes: hermesI18nJa,
      config: configI18nJa,
      claw3d: claw3dI18nJa,
    },
  },
} satisfies Resource;

function readKey(node: unknown, path: string): string | undefined {
  const result = path.split(".").reduce<unknown>((current, part) => {
    if (!current || typeof current !== "object") return undefined;
    return (current as Record<string, unknown>)[part];
  }, node);

  return typeof result === "string" ? result : undefined;
}

let locale: AppLocale = DEFAULT_ACTIVE_LOCALE;

export const sharedI18n = i18next.createInstance();

void sharedI18n.init({
  lng: locale,
  fallbackLng: FALLBACK_LOCALE,
  supportedLngs: APP_LOCALES,
  defaultNS: "translation",
  ns: ["translation"],
  interpolation: {
    escapeValue: false,
  },
  resources,
  initImmediate: false,
});

export function getLocale(): AppLocale {
  return locale;
}

export function setLocale(nextLocale: AppLocale): AppLocale {
  locale = nextLocale;
  void sharedI18n.changeLanguage(nextLocale);
  return locale;
}

export function t(
  key: string,
  lang: AppLocale = locale,
  options?: Record<string, unknown>,
): string {
  const translated = readKey(resources[lang]?.translation, key);
  const fallback = readKey(resources[FALLBACK_LOCALE].translation, key);
  const base = translated ?? fallback ?? key;

  if (!options) return base;

  return Object.entries(options).reduce((message, [name, value]) => {
    return message.replaceAll(`{{${name}}}`, String(value));
  }, base);
}

export { APP_LOCALES, DEFAULT_ACTIVE_LOCALE, FALLBACK_LOCALE, SOURCE_LOCALE };
export type { AppLocale };
