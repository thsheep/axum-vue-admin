<template>
  <AppPage :show-footer="true" bg-cover :style="{ backgroundImage: `url(${bgImg})` }">
    <div
      style="transform: translateY(25px)"
      class="m-auto max-w-1500 min-w-345 f-c-c rounded-10 bg-white bg-opacity-60 p-15 card-shadow"
      dark:bg-dark
    >
      <div hidden w-380 px-20 py-35 md:block>
        <icon-custom-front-page pt-10 text-300 color-primary></icon-custom-front-page>
      </div>

      <div w-320 flex-col px-20 py-35>
        <h5 f-c-c text-24 font-normal color="#6a6a6a">
          <icon-custom-logo mr-10 text-50 color-primary />{{ $t('app_name') }}
        </h5>
        <div mt-30>
          <n-input
            v-model:value="loginInfo.username"
            autofocus
            class="h-50 items-center pl-10 text-16"
            placeholder="admin"
            :maxlength="20"
          />
        </div>
        <div mt-30>
          <n-input
            v-model:value="loginInfo.password"
            class="h-50 items-center pl-10 text-16"
            type="password"
            show-password-on="mousedown"
            placeholder="123456"
            :maxlength="20"
            @keypress.enter="handleLogin"
          />
        </div>

        <div mt-20>
          <n-button
            h-50
            w-full
            rounded-5
            text-16
            type="primary"
            :loading="loading"
            @click="handleLogin"
          >
            {{ $t('views.login.text_login') }}
          </n-button>
        </div>
        <div class="mt-20 w-full flex items-start justify-end">
          <router-link to="/forgot-password">
            <n-button type="warning" text tag="a">{{ $t('views.login.forget_password') }}</n-button>
          </router-link>
        </div>
      </div>
    </div>
  </AppPage>
</template>

<script setup>
import bgImg from '@/assets/images/login_bg.webp'
import { useI18n } from 'vue-i18n'
import AppPage from "@/components/page/AppPage.vue";
import {useUserStore} from "@/store/modules/user";
import {useRoute, useRouter} from "vue-router";


const { t } = useI18n({ useScope: 'global' })
const router = useRouter()
const { query } = useRoute()

const loginInfo = ref({
  username: '',
  password: '',
})

const loading = ref(false)
async function handleLogin() {
  console.log(loginInfo.value)
  const { username, password } = loginInfo.value
  if (!username || !password) {
    $message.warning(t('views.login.message_input_username_password'))
    return
  }
  try {
    loading.value = true
    const userStore = useUserStore()
    await userStore.login({ username, password: password.toString() })
    if (query.redirect) {
      const path = query.redirect
      Reflect.deleteProperty(query, 'redirect')
      await router.push({path, query})
    } else {
      await router.push('/')
    }
  } catch (e) {
    console.error('login error', e.error)
  }
  loading.value = false
}
</script>
