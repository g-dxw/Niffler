export * from 'vue'

import { createApp as createVueApp } from 'vue'
import { i18n } from '@/i18n'

const createAppWithI18n = (...args: Parameters<typeof createVueApp>) => {
  const app = createVueApp(...args)
  app.use(i18n)
  return app
}

export const createApp = createAppWithI18n as typeof createVueApp
