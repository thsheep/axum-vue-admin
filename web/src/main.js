/** 重置样式 */
import '@/styles/reset.css'
import 'uno.css'
import '@/styles/global.scss'

import { createApp } from 'vue'
import { setupRouter } from '@/router'
import { createPinia } from 'pinia'
import App from './App.vue'
import { setupDirectives } from './directives'
import { useResize } from '@/utils'
import i18n from '@/i18n'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

async function setupApp() {
  const app = createApp(App)

  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)
  app.use(pinia)
  await setupRouter(app)
  setupDirectives(app)
  app.use(useResize)
  app.use(i18n)
  app.mount('#app')
}

setupApp()
