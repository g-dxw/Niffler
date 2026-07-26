<template>
  <div class="min-h-screen flex items-center justify-center px-6">
    <Card class="w-full max-w-md p-6 space-y-2">
      <h1 class="text-lg font-semibold text-foreground">
        {{ t('authCallback.processing') }}
      </h1>
      <p class="text-sm text-muted-foreground">
        {{ hint }}
      </p>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import Card from '@/components/ui/card.vue'
import apiClient from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/composables/useToast'
import { useI18n } from 'vue-i18n'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const { success, error: showError } = useToast()
const { t } = useI18n()

const hint = ref(t('authCallback.pleaseWait'))

function consumeRedirectPath(): string | null {
  const redirectPath = sessionStorage.getItem('redirectPath')
  if (redirectPath) {
    sessionStorage.removeItem('redirectPath')
    return redirectPath
  }
  return null
}

function clearUrlState() {
  // 清理 fragment，避免刷新时重复处理
  // 同时清理 query（oauth_bound / error_code / error_detail）
  const newUrl = window.location.pathname
  window.history.replaceState({}, document.title, newUrl)
}

function errorMessageFromCode(code: string): string {
  const map: Record<string, string> = {
    authorization_denied: t('authCallback.errors.authorizationDenied'), provider_disabled: t('authCallback.errors.providerDisabled'), provider_unavailable: t('authCallback.errors.providerUnavailable'), invalid_callback: t('authCallback.errors.invalidCallback'), invalid_state: t('authCallback.errors.invalidState'), token_exchange_failed: t('authCallback.errors.tokenExchangeFailed'), userinfo_fetch_failed: t('authCallback.errors.userInfoFetchFailed'), email_exists_local: t('authCallback.errors.emailExistsLocal'), email_is_ldap: t('authCallback.errors.emailIsLdap'), email_is_oauth: t('authCallback.errors.emailIsOauth'), registration_disabled: t('authCallback.errors.registrationDisabled'), oauth_already_bound: t('authCallback.errors.oauthAlreadyBound'), already_bound_provider: t('authCallback.errors.alreadyBoundProvider'), last_oauth_binding: t('authCallback.errors.lastOauthBinding'), last_login_method: t('authCallback.errors.lastLoginMethod'), ldap_no_oauth: t('authCallback.errors.ldapNoOauth'),
  }
  return map[code] || t('authCallback.errors.generic')
}

onMounted(async () => {
  // 1) 绑定成功提示
  const oauthBound = route.query.oauth_bound
  if (typeof oauthBound === 'string' && oauthBound) {
    success(t('authCallback.bound', { provider: oauthBound }))
    clearUrlState()
    const redirectPath = consumeRedirectPath()
    await router.replace(redirectPath || '/dashboard/settings')
    return
  }

  // 2) 错误提示
  const errorCode = route.query.error_code
  if (typeof errorCode === 'string' && errorCode) {
    showError(errorMessageFromCode(errorCode))
    clearUrlState()
    const redirectPath = consumeRedirectPath()
    await router.replace(redirectPath || '/')
    return
  }

  // 3) 登录成功：解析 fragment token
  const hash = window.location.hash.startsWith('#') ? window.location.hash.slice(1) : window.location.hash
  const params = new URLSearchParams(hash)
  const accessToken = params.get('access_token')

  clearUrlState()

  if (!accessToken) {
    showError(t('authCallback.missingToken'))
    await router.replace('/')
    return
  }

  hint.value = t('authCallback.writingSession')
  apiClient.setToken(accessToken)

  authStore.syncToken()

  hint.value = t('authCallback.fetchingUser')
  await authStore.fetchCurrentUser()

  success(t('authCallback.loginSuccess'))

  const redirectPath = consumeRedirectPath()
  const target = redirectPath || (authStore.canAccessAdmin ? '/admin/dashboard' : '/dashboard')
  await router.replace(target)
})
</script>
