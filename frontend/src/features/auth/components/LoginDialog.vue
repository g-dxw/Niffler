<template>
  <Dialog
    v-model="isOpen"
    size="md"
    no-padding
  >
    <div class="px-6 py-6 sm:px-8 sm:py-8">
      <div class="mb-7 flex flex-col items-center text-center">
        <img
          src="/aether_adaptive.svg"
          :alt="siteName"
          class="mb-4 h-14 w-14"
        >
        <h2 class="text-2xl font-semibold text-foreground">
          {{ dialogTitle }}
        </h2>
      </div>

      <div
        v-if="authPanel === 'login'"
        class="space-y-5"
      >
        <AuthDemoAccounts
          v-if="isDemo"
          :disabled="authStore.loading"
          @fill="fillDemoAccount"
        />

        <OAuthLoginOptions
          :providers="oauthProviders"
          :disabled="authStore.loading"
          @login="handleOAuthLogin"
        />

        <div
          v-if="oauthProviders.length > 0"
          class="flex items-center gap-3"
        >
          <div class="h-px flex-1 bg-border" />
          <span class="text-xs text-muted-foreground">账号密码</span>
          <div class="h-px flex-1 bg-border" />
        </div>

        <AuthTypeSwitch
          v-model="authType"
          :local-enabled="localEnabled"
          :ldap-enabled="ldapEnabled"
          :ldap-exclusive="ldapExclusive"
          :disabled="authStore.loading"
        />

        <form
          ref="loginFormEl"
          name="login"
          action="/api/auth/login"
          method="post"
          class="space-y-4"
          autocomplete="on"
          data-form-type="login"
          @submit.prevent="handleLogin"
        >
          <div class="space-y-1.5">
            <Label
              for="username"
              class="text-sm"
            >
              {{ emailLabel }}
            </Label>
            <Input
              id="username"
              v-model="form.email"
              type="text"
              name="username"
              required
              placeholder="用户名或邮箱"
              autocomplete="username"
              autocapitalize="none"
              spellcheck="false"
              :disable-autofill="false"
            />
          </div>

          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <Label
                for="password"
                class="text-sm"
              >
                密码
              </Label>
              <button
                v-if="showPasswordResetLink"
                type="button"
                class="text-xs font-medium text-primary transition-colors hover:text-foreground active:translate-y-px disabled:pointer-events-none disabled:opacity-50"
                :disabled="authStore.loading"
                :data-state="authStore.loading ? 'disabled' : 'idle'"
                @click="openPasswordResetPanel"
              >
                忘记密码？
              </button>
            </div>
            <Input
              id="password"
              v-model="form.password"
              type="password"
              name="password"
              required
              placeholder="输入密码"
              autocomplete="current-password"
              :disable-autofill="false"
            />
          </div>

          <Button
            type="submit"
            :disabled="authStore.loading"
            class="h-12 w-full"
          >
            {{ authStore.loading ? '登录中...' : '登录' }}
          </Button>
        </form>

        <AuthRegisterPrompt
          :allow-registration="allowRegistration"
          :show-contact-admin="!isDemo"
          :disabled="authStore.loading"
          @register="handleSwitchToRegister"
        />
      </div>

      <PasswordResetRequestPanel
        v-else
        v-model:email="passwordResetEmail"
        :loading="passwordResetLoading"
        :notice="passwordResetNotice"
        @submit="handlePasswordResetRequest"
        @back="authPanel = 'login'"
      />
    </div>
  </Dialog>

  <!-- Register Dialog -->
  <RegisterDialog
    v-model:open="showRegisterDialog"
    :require-email-verification="requireEmailVerification"
    :email-configured="emailConfigured"
    :password-policy-level="passwordPolicyLevel"
    :turnstile-enabled="turnstileEnabled"
    :turnstile-site-key="turnstileSiteKey"
    :privacy-policy="privacyPolicy"
    @success="handleRegisterSuccess"
    @switch-to-login="handleSwitchToLogin"
  />
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Dialog } from '@/components/ui'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/composables/useToast'
import { useSiteInfo } from '@/composables/useSiteInfo'
import { normalizePasswordPolicyLevel, type PasswordPolicyLevel } from '@/utils/passwordPolicy'
import { isDemoMode, DEMO_ACCOUNTS } from '@/config/demo'
import RegisterDialog from './RegisterDialog.vue'
import AuthDemoAccounts from './AuthDemoAccounts.vue'
import AuthRegisterPrompt from './AuthRegisterPrompt.vue'
import AuthTypeSwitch from './AuthTypeSwitch.vue'
import OAuthLoginOptions from './OAuthLoginOptions.vue'
import PasswordResetRequestPanel from './PasswordResetRequestPanel.vue'
import { authApi, type RegistrationPrivacyPolicySettings } from '@/api/auth'
import { oauthApi, type OAuthProviderInfo } from '@/api/oauth'
import { getClientDeviceId } from '@/utils/deviceId'
import { getApiUrl } from '@/utils/url'
import { getErrorMessage } from '@/types/api-error'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const { success: showSuccess, warning: showWarning, error: showError } = useToast()
const { siteName } = useSiteInfo()

const isOpen = ref(props.modelValue)
const isDemo = computed(() => isDemoMode())
const authPanel = ref<'login' | 'passwordReset'>('login')
const showRegisterDialog = ref(false)
const requireEmailVerification = ref(false)
const emailConfigured = ref(true) // 邮箱服务是否已配置
const passwordPolicyLevel = ref<PasswordPolicyLevel>('weak')
const turnstileEnabled = ref(false)
const turnstileSiteKey = ref<string | null>(null)
const allowRegistration = ref(false) // 由系统配置控制，默认关闭
const privacyPolicy = ref<RegistrationPrivacyPolicySettings>({
  enabled: false,
  format: 'markdown',
  content: '',
  version: ''
})

// LDAP authentication settings
const PREFERRED_AUTH_TYPE_KEY = 'aether_preferred_auth_type'
function getStoredAuthType(): 'local' | 'ldap' {
  const stored = localStorage.getItem(PREFERRED_AUTH_TYPE_KEY)
  return (stored === 'ldap' || stored === 'local') ? stored : 'local'
}
const authType = ref<'local' | 'ldap'>(getStoredAuthType())
const localEnabled = ref(true)
const ldapEnabled = ref(false)
const ldapExclusive = ref(false)

const oauthProviders = ref<OAuthProviderInfo[]>([])
const loginFormEl = ref<HTMLFormElement | null>(null)
const passwordResetEmail = ref('')
const passwordResetLoading = ref(false)
const passwordResetNotice = ref('')
const passwordResetRequestSuccessText = '如果该邮箱存在，会收到重置密码邮件'

// 保存用户的认证类型偏好
watch(authType, (newType) => {
  localStorage.setItem(PREFERRED_AUTH_TYPE_KEY, newType)
})

const emailLabel = computed(() => {
  return '用户名/邮箱'
})

const dialogTitle = computed(() => {
  return authPanel.value === 'passwordReset' ? '找回密码' : `登录到 ${siteName.value}`
})

const showPasswordResetLink = computed(() => {
  return localEnabled.value && authType.value === 'local'
})

watch(() => props.modelValue, (val) => {
  isOpen.value = val
  // 打开对话框时重置表单
  if (val) {
    authPanel.value = 'login'
    passwordResetEmail.value = ''
    passwordResetNotice.value = ''
    form.value = {
      email: '',
      password: ''
    }
  }
})

watch(isOpen, (val) => {
  emit('update:modelValue', val)
})

const form = ref({
  email: '',
  password: ''
})

function fillDemoAccount(type: 'admin' | 'user') {
  const account = DEMO_ACCOUNTS[type]
  form.value.email = account.email
  form.value.password = account.password
}

async function handleLogin(event?: Event) {
  const { email, password } = readCurrentLoginCredentials(event)

  if (!email || !password) {
    showWarning('请输入邮箱和密码')
    return
  }

  const success = await authStore.login(email, password, authType.value)
  if (success) {
    const targetPath = consumeStoredRedirectPath() ?? (authStore.canAccessAdmin ? '/admin/dashboard' : '/dashboard')

    try {
      const navigationFailure = await router.push(targetPath)
      if (navigationFailure) {
        throw navigationFailure
      }
    } catch {
      showError('登录成功，但跳转失败，请刷新页面或手动进入控制台')
      return
    }

    showSuccess('登录成功，正在跳转...')

    // 关闭对话框
    isOpen.value = false
  } else {
    showError(authStore.error || '登录失败，请检查邮箱和密码')
  }
}

function readCurrentLoginCredentials(event?: Event): { email: string; password: string } {
  const formElement = event?.currentTarget instanceof HTMLFormElement
    ? event.currentTarget
    : loginFormEl.value

  const emailInput = formElement?.elements.namedItem('username')
  const passwordInput = formElement?.elements.namedItem('password')

  const email = emailInput instanceof HTMLInputElement
    ? emailInput.value.trim()
    : form.value.email.trim()
  const password = passwordInput instanceof HTMLInputElement
    ? passwordInput.value
    : form.value.password

  form.value.email = email
  form.value.password = password

  return { email, password }
}

function consumeStoredRedirectPath(): string | null {
  const redirectPath = sessionStorage.getItem('redirectPath')
  if (redirectPath) {
    sessionStorage.removeItem('redirectPath')
  }
  if (!redirectPath || redirectPath === '/' || !redirectPath.startsWith('/') || redirectPath.startsWith('//')) {
    return null
  }
  return redirectPath
}

function handleOAuthLogin(providerType: string) {
  // 如果 sessionStorage 中没有 redirectPath（用户直接点击登录而非被守卫拦截），
  // 则不设置，让 AuthCallback 使用默认跳转逻辑
  const authorizeUrl = new URL(
    getApiUrl(`/api/oauth/${providerType}/authorize`),
    window.location.origin,
  )
  authorizeUrl.searchParams.set('client_device_id', getClientDeviceId())
  window.location.href = authorizeUrl.toString()
}

function handleSwitchToRegister() {
  isOpen.value = false
  showRegisterDialog.value = true
}

function openPasswordResetPanel() {
  const { email } = readCurrentLoginCredentials()
  passwordResetEmail.value = email.includes('@') ? email : ''
  passwordResetNotice.value = ''
  authPanel.value = 'passwordReset'
}

async function handlePasswordResetRequest() {
  const email = passwordResetEmail.value.trim()
  if (!email) {
    showWarning('请输入邮箱')
    return
  }
  passwordResetLoading.value = true
  passwordResetNotice.value = ''
  try {
    const response = await authApi.requestPasswordReset(email)
    passwordResetNotice.value = response.message || passwordResetRequestSuccessText
    showSuccess('申请已提交，请查看邮箱')
  } catch (error) {
    showError(getErrorMessage(error, '发送失败，请稍后重试'))
  } finally {
    passwordResetLoading.value = false
  }
}

function handleRegisterSuccess() {
  showRegisterDialog.value = false
  showSuccess('注册成功！请登录')
  isOpen.value = true
}

function handleSwitchToLogin() {
  showRegisterDialog.value = false
  isOpen.value = true
}

// Load authentication and registration settings on mount
onMounted(async () => {
  try {
    const [regSettings, authSettings, providers] = await Promise.all([
      authApi.getRegistrationSettings(),
      authApi.getAuthSettings(),
      oauthApi.getProviders().catch(() => []),
    ])

    allowRegistration.value = !!regSettings.enable_registration
    requireEmailVerification.value = !!regSettings.require_email_verification
    emailConfigured.value = !!regSettings.email_configured
    passwordPolicyLevel.value = normalizePasswordPolicyLevel(regSettings.password_policy_level)
    turnstileEnabled.value = !!regSettings.turnstile_enabled
    turnstileSiteKey.value = regSettings.turnstile_site_key || null
    privacyPolicy.value = regSettings.privacy_policy ?? {
      enabled: false,
      format: 'markdown',
      content: '',
      version: ''
    }

    localEnabled.value = authSettings.local_enabled
    ldapEnabled.value = authSettings.ldap_enabled
    ldapExclusive.value = authSettings.ldap_exclusive
    // 若仅允许 LDAP 登录，则禁用本地注册入口
    if (ldapExclusive.value) {
      allowRegistration.value = false
    }

    // Set default auth type based on settings
    if (authSettings.ldap_exclusive) {
      authType.value = 'ldap'
    } else if (!authSettings.local_enabled && authSettings.ldap_enabled) {
      authType.value = 'ldap'
    } else {
      authType.value = 'local'
    }

    oauthProviders.value = providers
    if (allowRegistration.value && (route.path === '/register' || typeof route.query.invite === 'string')) {
      isOpen.value = false
      showRegisterDialog.value = true
    }
  } catch {
    // If获取失败，保持默认：关闭注册 & 关闭邮箱验证 & 使用本地认证
    allowRegistration.value = false
    requireEmailVerification.value = false
    emailConfigured.value = false
    passwordPolicyLevel.value = 'weak'
    turnstileEnabled.value = false
    turnstileSiteKey.value = null
    privacyPolicy.value = {
      enabled: false,
      format: 'markdown',
      content: '',
      version: ''
    }
    localEnabled.value = true
    ldapEnabled.value = false
    ldapExclusive.value = false
    authType.value = 'local'
    oauthProviders.value = []
  }
})
</script>
