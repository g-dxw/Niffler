<template>
  <Dialog
    v-model:open="isOpen"
    size="lg"
  >
    <div class="space-y-6">
      <!-- Logo 和标题 -->
      <div class="flex flex-col items-center text-center">
        <div class="mb-4 rounded-3xl border border-primary/30 dark:border-[#cc785c]/30 bg-primary/5 dark:bg-transparent p-4 shadow-inner shadow-white/40 dark:shadow-[#cc785c]/10">
          <img
            src="/niffler-logo.svg"
            alt="Logo"
            class="h-16 w-16 dark:invert"
          >
        </div>
        <h2 class="text-2xl font-semibold text-slate-900 dark:text-white">
          {{ t('registerDialog.title') }}
        </h2>
        <p class="mt-1 text-sm text-muted-foreground">
          {{ emailConfigured ? t('registerDialog.descriptionWithEmail') : t('registerDialog.descriptionWithoutEmail') }}
        </p>
      </div>

      <!-- 注册表单 -->
      <form
        class="space-y-4"
        autocomplete="off"
        data-form-type="other"
        @submit.prevent="handleSubmit"
      >
        <!-- Email (仅当邮箱服务已配置时显示) -->
        <div
          v-if="emailConfigured"
          class="space-y-2"
        >
          <Label for="reg-email">
            {{ t('registerDialog.email') }}
            <span
              v-if="requireEmailVerification"
              class="text-destructive"
            >*</span>
            <span
              v-else
              class="text-muted-foreground text-xs"
            >{{ t('registerDialog.optional') }}</span>
          </Label>
          <Input
            id="reg-email"
            v-model="formData.email"
            type="email"
            placeholder="hello@example.com"
            :required="requireEmailVerification"
            disable-autofill
            :disabled="isLoading || emailVerified"
          />
        </div>

        <div
          v-if="turnstileRequired"
          class="space-y-2"
        >
          <Label>{{ t('registerDialog.captcha') }} <span class="text-destructive">*</span></Label>
          <TurnstileWidget
            ref="turnstileWidgetRef"
            v-model="turnstileToken"
            :site-key="turnstileSiteKey"
            :action="currentTurnstileAction"
            :disabled="isLoading || isSendingCode"
            @error="handleTurnstileError"
          />
        </div>

        <!-- Verification Code Section (仅当需要邮箱验证时显示) -->
        <div
          v-if="emailConfigured && requireEmailVerification"
          class="space-y-3"
        >
          <div class="flex items-center justify-between">
            <Label>{{ t('registerDialog.verificationCode') }} <span class="text-destructive">*</span></Label>
            <Button
              type="button"
              variant="link"
              size="sm"
              class="h-auto p-0 text-xs"
              :disabled="isSendingCode || !canSendCode || emailVerified"
              @click="handleSendCode"
            >
              {{ sendCodeButtonText }}
            </Button>
          </div>
          <div class="flex justify-center gap-2">
            <!-- 发送中显示 loading -->
            <div
              v-if="isSendingCode"
              class="flex items-center justify-center gap-2 h-14 text-muted-foreground"
            >
              <svg
                class="animate-spin h-5 w-5"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                />
                <path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              <span class="text-sm">
                {{ sendCodeLoadingText }}
              </span>
            </div>
            <!-- 验证码输入框 -->
            <template v-else>
              <input
                v-for="(_, index) in 6"
                :key="index"
                :ref="(el) => setCodeInputRef(index, el as HTMLInputElement)"
                v-model="codeDigits[index]"
                type="text"
                inputmode="numeric"
                maxlength="1"
                autocomplete="off"
                data-form-type="other"
                class="w-12 h-14 text-center text-xl font-semibold border-2 rounded-lg bg-background transition-all focus:outline-none focus:ring-2 focus:ring-primary/20"
                :class="verificationError ? 'border-destructive' : 'border-border focus:border-primary'"
                :disabled="emailVerified"
                @input="handleCodeInput(index, $event)"
                @keydown="handleCodeKeyDown(index, $event)"
                @paste="handleCodePaste"
              >
            </template>
          </div>
        </div>

        <!-- Username -->
        <div class="space-y-2">
          <Label for="reg-uname">{{ t('registerDialog.username') }} <span class="text-destructive">*</span></Label>
          <Input
            id="reg-uname"
            v-model="formData.username"
            type="text"
            :placeholder="t('registerDialog.usernamePlaceholder')"
            required
            disable-autofill
            :disabled="isLoading"
            :class="usernameError ? 'border-destructive' : ''"
          />
          <p
            v-if="usernameError"
            class="text-xs text-destructive"
          >
            {{ usernameError }}
          </p>
        </div>

        <!-- Password -->
        <div class="space-y-2">
          <Label :for="`pwd-${formNonce}`">{{ t('registerDialog.password') }} <span class="text-destructive">*</span></Label>
          <Input
            :id="`pwd-${formNonce}`"
            v-model="formData.password"
            masked
            autocomplete="new-password"
            disable-autofill
            :name="`pwd-${formNonce}`"
            :placeholder="getPasswordPolicyPlaceholder(props.passwordPolicyLevel, translatePasswordPolicy)"
            required
            :disabled="isLoading"
          />
          <p
            v-if="passwordError"
            class="text-xs text-destructive"
          >
            {{ passwordError }}
          </p>
          <p
            v-else
            class="text-xs text-muted-foreground"
          >
            {{ passwordHint }}
          </p>
        </div>

        <!-- Confirm Password -->
        <div class="space-y-2">
          <Label :for="`pwd-confirm-${formNonce}`">{{ t('registerDialog.confirmPassword') }} <span class="text-destructive">*</span></Label>
          <Input
            :id="`pwd-confirm-${formNonce}`"
            v-model="formData.confirmPassword"
            masked
            autocomplete="new-password"
            disable-autofill
            :name="`pwd-confirm-${formNonce}`"
            :placeholder="t('registerDialog.confirmPasswordPlaceholder')"
            required
            :disabled="isLoading"
          />
          <p
            v-if="formData.confirmPassword && formData.password !== formData.confirmPassword"
            class="text-xs text-destructive"
          >
            {{ t('registerDialog.passwordMismatch') }}
          </p>
        </div>

        <div
          v-if="inviteCode"
          class="rounded-lg border border-primary/20 bg-primary/5 px-3 py-2 text-xs text-muted-foreground"
        >
          {{ t('registerDialog.inviteRecognized') }} <span class="font-mono font-semibold text-foreground">{{ inviteCode }}</span>
        </div>

        <div
          v-if="privacyPolicyEnabled"
          class="rounded-lg border border-border bg-muted/30 p-3"
        >
          <label class="flex items-start gap-2 text-sm">
            <Checkbox
              :checked="privacyAccepted"
              class="mt-0.5"
              @update:checked="privacyAccepted = !!$event"
            />
            <span class="leading-6">
              {{ t('registerDialog.agreePrefix') }}
              <button
                type="button"
                class="font-medium text-primary underline-offset-4 hover:underline"
                @click="privacyDialogOpen = true"
              >
                {{ t('registerDialog.privacyPolicy') }}
              </button>
              <RouterLink
                to="/privacy-policy"
                target="_blank"
                class="ml-1 text-xs text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
              >
                {{ t('registerDialog.openNewWindow') }}
              </RouterLink>
            </span>
          </label>
          <p class="mt-2 text-xs text-muted-foreground">
            {{ t('registerDialog.currentVersion', { version: privacyPolicyVersion }) }}
          </p>
        </div>
      </form>

      <!-- 登录链接 -->
      <div class="text-center text-sm">
        {{ t('registerDialog.hasAccount') }}
        <Button
          variant="link"
          class="h-auto p-0"
          @click="handleSwitchToLogin"
        >
          {{ t('registerDialog.signInNow') }}
        </Button>
      </div>
    </div>

    <template #footer>
      <Button
        type="button"
        variant="outline"
        class="w-full sm:w-auto border-slate-200 dark:border-slate-600 text-slate-500 dark:text-slate-400 hover:text-primary hover:border-primary/50 hover:bg-primary/5 dark:hover:text-primary dark:hover:border-primary/50 dark:hover:bg-primary/10"
        :disabled="isLoading"
        @click="handleCancel"
      >
        {{ t('common.cancel') }}
      </Button>
      <Button
        class="w-full sm:w-auto bg-primary hover:bg-primary/90 text-white border-0"
        :disabled="isLoading || !canSubmit"
        @click="handleSubmit"
      >
        {{ isLoading ? loadingText : t('registerDialog.register') }}
      </Button>
    </template>
  </Dialog>

  <Dialog
    v-model="privacyDialogOpen"
    size="2xl"
    :title="t('registerDialog.privacyPolicy')"
  >
    <!-- eslint-disable vue/no-v-html -->
    <div
      class="prose prose-sm dark:prose-invert max-h-[60vh] max-w-none overflow-y-auto"
      v-html="renderedPrivacyPolicy"
    />
    <!-- eslint-enable vue/no-v-html -->
    <template #footer>
      <Button
        type="button"
        @click="privacyDialogOpen = false"
      >
        {{ t('registerDialog.gotIt') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { marked } from 'marked'
import { authApi, type RegisterRequest, type RegistrationPrivacyPolicySettings } from '@/api/auth'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { sanitizeHtml, sanitizeMarkdown } from '@/utils/sanitize'
import {
  getPasswordPolicyHint,
  getPasswordPolicyPlaceholder,
  validatePasswordByPolicy,
  type PasswordPolicyLevel,
} from '@/utils/passwordPolicy'
import { Dialog } from '@/components/ui'
import Button from '@/components/ui/button.vue'
import Checkbox from '@/components/ui/checkbox.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import TurnstileWidget from './TurnstileWidget.vue'

const INVITE_CODE_STORAGE_KEY = 'aether_invite_code'

interface Props {
  open?: boolean
  requireEmailVerification?: boolean
  emailConfigured?: boolean
  passwordPolicyLevel?: PasswordPolicyLevel
  turnstileEnabled?: boolean
  turnstileSiteKey?: string | null
  privacyPolicy?: RegistrationPrivacyPolicySettings
}

interface Emits {
  (e: 'update:open', value: boolean): void
  (e: 'success'): void
  (e: 'switchToLogin'): void
}

const props = withDefaults(defineProps<Props>(), {
  open: false,
  requireEmailVerification: false,
  emailConfigured: true,
  passwordPolicyLevel: 'weak',
  turnstileEnabled: false,
  turnstileSiteKey: null,
  privacyPolicy: () => ({
    enabled: false,
    format: 'markdown',
    content: '',
    version: ''
  })
})

const emit = defineEmits<Emits>()
const { t } = useI18n()
const { success, error: showError } = useToast()
const translatePasswordPolicy = (key: string, params?: Record<string, string | number>) => t(key, params ?? {})

// Form nonce for password fields (prevent autofill)
const formNonce = ref(createFormNonce())

function createFormNonce(): string {
  return Math.random().toString(36).slice(2, 10)
}

// Verification code inputs
const codeInputRefs = ref<(HTMLInputElement | null)[]>([])
const codeDigits = ref<string[]>(['', '', '', '', '', ''])

const setCodeInputRef = (index: number, el: HTMLInputElement | null) => {
  codeInputRefs.value[index] = el
}

// Handle verification code input
const handleCodeInput = (index: number, event: Event) => {
  const input = event.target as HTMLInputElement
  const value = input.value

  // Only allow digits
  if (!/^\d*$/.test(value)) {
    input.value = codeDigits.value[index]
    return
  }

  codeDigits.value[index] = value

  // Auto-focus next input
  if (value && index < 5) {
    codeInputRefs.value[index + 1]?.focus()
  }

  // Check if all digits are filled
  const fullCode = codeDigits.value.join('')
  if (fullCode.length === 6 && /^\d+$/.test(fullCode)) {
    handleCodeComplete(fullCode)
  }
}

const handleCodeKeyDown = (index: number, event: KeyboardEvent) => {
  // Handle backspace
  if (event.key === 'Backspace') {
    if (!codeDigits.value[index] && index > 0) {
      // If current input is empty, move to previous and clear it
      codeInputRefs.value[index - 1]?.focus()
      codeDigits.value[index - 1] = ''
    } else {
      // Clear current input
      codeDigits.value[index] = ''
    }
  }
  // Handle arrow keys
  else if (event.key === 'ArrowLeft' && index > 0) {
    codeInputRefs.value[index - 1]?.focus()
  } else if (event.key === 'ArrowRight' && index < 5) {
    codeInputRefs.value[index + 1]?.focus()
  }
}

const handleCodePaste = (event: ClipboardEvent) => {
  event.preventDefault()
  const pastedData = event.clipboardData?.getData('text') || ''
  const cleanedData = pastedData.replace(/\D/g, '').slice(0, 6)

  if (cleanedData) {
    // Fill digits
    for (let i = 0; i < 6; i++) {
      codeDigits.value[i] = cleanedData[i] || ''
    }

    // Focus the next empty input or the last input
    const nextEmptyIndex = codeDigits.value.findIndex((d) => !d)
    const focusIndex = nextEmptyIndex >= 0 ? nextEmptyIndex : 5
    codeInputRefs.value[focusIndex]?.focus()

    // Check if all digits are filled
    if (cleanedData.length === 6) {
      handleCodeComplete(cleanedData)
    }
  }
}

const clearCodeInputs = () => {
  codeDigits.value = ['', '', '', '', '', '']
  codeInputRefs.value[0]?.focus()
}

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit('update:open', value)
})

const formData = ref({
  email: '',
  username: '',
  password: '',
  confirmPassword: '',
  verificationCode: ''
})

const isLoading = ref(false)
const loadingText = ref(t('registerDialog.registering'))
const isSendingCode = ref(false)
const emailVerified = ref(false)
const verificationError = ref(false)
const codeSentAt = ref<number | null>(null)
const cooldownSeconds = ref(0)
const expireMinutes = ref(5)
const cooldownTimer = ref<number | null>(null)
type TurnstileAction = 'send_verification_code' | 'register'
const turnstileToken = ref('')
const turnstileWidgetRef = ref<InstanceType<typeof TurnstileWidget> | null>(null)

const turnstileSiteKey = computed(() => props.turnstileSiteKey || '')
const turnstileRequired = computed(() => !!props.turnstileEnabled && !!turnstileSiteKey.value)
const currentTurnstileAction = computed<TurnstileAction>(() =>
  props.requireEmailVerification && !emailVerified.value
    ? 'send_verification_code'
    : 'register'
)

const resetTurnstile = () => {
  turnstileToken.value = ''
  turnstileWidgetRef.value?.reset()
}

const handleTurnstileError = (message: string) => {
  showError(message, t('registerDialog.captchaFailed'))
}

const inviteCode = ref<string | null>(null)
const privacyAccepted = ref(false)
const privacyDialogOpen = ref(false)
const privacyPolicyEnabled = computed(() => !!props.privacyPolicy?.enabled)
const privacyPolicyVersion = computed(() => props.privacyPolicy?.version || '1')
const renderedPrivacyPolicy = computed(() => {
  const policy = props.privacyPolicy
  if (!policy?.content) return `<p>${t('registerDialog.noPrivacyContent')}</p>`
  if (policy.format === 'html') {
    return sanitizeHtml(policy.content)
  }
  const rawHtml = marked(policy.content) as string
  return sanitizeMarkdown(rawHtml)
})

function loadInviteCode(): string | null {
  if (typeof window === 'undefined') return null
  const fromQuery = new URLSearchParams(window.location.search).get('invite')
  const normalized = (fromQuery || localStorage.getItem(INVITE_CODE_STORAGE_KEY) || '')
    .trim()
    .toUpperCase()
  if (!normalized) return null
  localStorage.setItem(INVITE_CODE_STORAGE_KEY, normalized)
  return normalized
}

// Send code cooldown timer
const canSendCode = computed(() => {
  if (!formData.value.email) return false
  if (cooldownSeconds.value > 0) return false
  if (
    turnstileRequired.value &&
    currentTurnstileAction.value === 'send_verification_code' &&
    !turnstileToken.value
  ) return false
  return true
})

const sendCodeButtonText = computed(() => {
  if (isSendingCode.value) return t('registerDialog.sending')
  if (emailVerified.value) return t('registerDialog.verified')
  if (cooldownSeconds.value > 0) return t('registerDialog.retryAfterSeconds', { seconds: cooldownSeconds.value })
  if (
    turnstileRequired.value &&
    currentTurnstileAction.value === 'send_verification_code' &&
    !turnstileToken.value
  ) return t('registerDialog.completeCaptchaFirst')
  if (codeSentAt.value) return t('registerDialog.resendCode')
  return t('registerDialog.sendCode')
})

const sendCodeLoadingText = computed(() => t('registerDialog.sendingCode'))

// 用户名验证
const usernameRegex = /^[a-zA-Z0-9_.-]+$/
const usernameError = computed(() => {
  const username = formData.value.username.trim()
  if (!username) return ''
  if (username.length < 3) return t('registerDialog.usernameTooShort')
  if (username.length > 30) return t('registerDialog.usernameTooLong')
  if (!usernameRegex.test(username)) return t('registerDialog.usernameInvalid')
  return ''
})

const passwordHint = computed(() => getPasswordPolicyHint(props.passwordPolicyLevel, translatePasswordPolicy))
const passwordError = computed(() =>
  validatePasswordByPolicy(formData.value.password, props.passwordPolicyLevel, translatePasswordPolicy)
)

const canSubmit = computed(() => {
  // 基本信息：用户名和密码必填
  const hasBasicInfo =
    formData.value.username &&
    formData.value.password &&
    formData.value.confirmPassword

  if (!hasBasicInfo) return false

  // 用户名格式验证
  if (usernameError.value) return false

  // 如果需要邮箱验证，邮箱和验证都必须完成
  if (props.requireEmailVerification) {
    if (!formData.value.email || !emailVerified.value) {
      return false
    }
  }

  if (
    turnstileRequired.value &&
    currentTurnstileAction.value === 'register' &&
    !turnstileToken.value
  ) {
    return false
  }

  // Check password match
  if (formData.value.password !== formData.value.confirmPassword) {
    return false
  }

  if (passwordError.value) {
    return false
  }

  if (privacyPolicyEnabled.value && !privacyAccepted.value) {
    return false
  }

  return true
})

// 查询并恢复验证状态
const checkAndRestoreVerificationStatus = async (email: string) => {
  if (!email || !props.requireEmailVerification) return

  try {
    const status = await authApi.getVerificationStatus(email)

    if (status.delivery_status === 'failed') {
      codeSentAt.value = null
      verificationError.value = true
      cooldownSeconds.value = 0
      if (cooldownTimer.value !== null) {
        clearInterval(cooldownTimer.value)
        cooldownTimer.value = null
      }
      showError(status.delivery_error || t('registerDialog.codeEmailFailed'))
      return
    }

    // 注意：不恢复 is_verified 状态
    // 刷新页面后需要重新发送验证码并验证，防止验证码被他人使用
    // 只恢复"有待验证验证码"的状态（冷却时间）
    if (status.has_pending_code) {
      codeSentAt.value = Date.now()
      verificationError.value = false

      // 恢复冷却时间
      if (status.cooldown_remaining && status.cooldown_remaining > 0) {
        startCooldown(status.cooldown_remaining)
      }
    }
  } catch {
    // 查询失败时静默处理，不影响用户体验
  }
}

// 邮箱查询防抖定时器
let emailCheckTimer: number | null = null

// 监听邮箱变化，查询验证状态
watch(
  () => formData.value.email,
  (newEmail, oldEmail) => {
    // 邮箱变化时重置验证状态
    if (newEmail !== oldEmail) {
      emailVerified.value = false
      verificationError.value = false
      codeSentAt.value = null
      cooldownSeconds.value = 0
      if (cooldownTimer.value !== null) {
        clearInterval(cooldownTimer.value)
        cooldownTimer.value = null
      }
      codeDigits.value = ['', '', '', '', '', '']
      resetTurnstile()
    }

    // 清除之前的定时器
    if (emailCheckTimer !== null) {
      clearTimeout(emailCheckTimer)
    }

    // 验证邮箱格式
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(newEmail)) return

    // 防抖：500ms 后查询验证状态
    emailCheckTimer = window.setTimeout(() => {
      checkAndRestoreVerificationStatus(newEmail)
    }, 500)
  }
)

watch(currentTurnstileAction, () => {
  resetTurnstile()
})

// Reset form when dialog opens
watch(isOpen, (newValue) => {
  if (newValue) {
    resetForm()
  }
})

// Start cooldown timer
const startCooldown = (seconds: number) => {
  // Clear existing timer if any
  if (cooldownTimer.value !== null) {
    clearInterval(cooldownTimer.value)
  }

  cooldownSeconds.value = seconds
  cooldownTimer.value = window.setInterval(() => {
    cooldownSeconds.value--
    if (cooldownSeconds.value <= 0) {
      if (cooldownTimer.value !== null) {
        clearInterval(cooldownTimer.value)
        cooldownTimer.value = null
      }
    }
  }, 1000)
}

// Cleanup timer on unmount
onUnmounted(() => {
  if (cooldownTimer.value !== null) {
    clearInterval(cooldownTimer.value)
  }
  if (emailCheckTimer !== null) {
    clearTimeout(emailCheckTimer)
  }
})

const resetForm = () => {
  formData.value = {
    email: '',
    username: '',
    password: '',
    confirmPassword: '',
    verificationCode: ''
  }
  emailVerified.value = false
  verificationError.value = false
  isSendingCode.value = false
  codeSentAt.value = null
  cooldownSeconds.value = 0
  inviteCode.value = loadInviteCode()
  privacyAccepted.value = false
  privacyDialogOpen.value = false

  // Reset password field nonce
  formNonce.value = createFormNonce()

  // Clear timer
  if (cooldownTimer.value !== null) {
    clearInterval(cooldownTimer.value)
    cooldownTimer.value = null
  }

  // Clear verification code inputs
  codeDigits.value = ['', '', '', '', '', '']
  resetTurnstile()
}

const handleSendCode = async () => {
  if (!formData.value.email) {
    showError(t('registerDialog.enterEmail'))
    return
  }

  // Basic email validation
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  if (!emailRegex.test(formData.value.email)) {
    showError(t('registerDialog.enterValidEmail'), t('registerDialog.emailFormatError'))
    return
  }

  isSendingCode.value = true

  try {
    const response = await authApi.sendVerificationCode(
      formData.value.email,
      turnstileRequired.value ? turnstileToken.value : undefined
    )

    if (response.success) {
      resetTurnstile()
      codeSentAt.value = Date.now()
      if (response.expire_minutes) {
        expireMinutes.value = response.expire_minutes
      }

      success(
        response.message || t('registerDialog.codeSendingNotice', { minutes: expireMinutes.value }),
        t('registerDialog.codeSendingTitle')
      )

      // Start 60 second cooldown
      startCooldown(60)

      // Focus the first verification code input
      nextTick(() => {
        codeInputRefs.value[0]?.focus()
      })
    } else {
      resetTurnstile()
      showError(response.message || t('registerDialog.tryAgainLater'), t('registerDialog.sendFailed'))
    }
  } catch (error: unknown) {
    resetTurnstile()
    showError(parseApiError(error, t('registerDialog.networkError')), t('registerDialog.sendFailed'))
  } finally {
    isSendingCode.value = false
    resetTurnstile()
  }
}

const handleCodeComplete = async (code: string) => {
  if (!formData.value.email || code.length !== 6) return

  // 如果已经验证成功，不再重复验证
  if (emailVerified.value) return

  isLoading.value = true
  loadingText.value = t('registerDialog.verifying')
  verificationError.value = false

  try {
    const response = await authApi.verifyEmail(formData.value.email, code)

    if (response.success) {
      emailVerified.value = true
      success(t('registerDialog.emailVerifiedContinue'), t('registerDialog.verified'))
    } else {
      verificationError.value = true
      showError(response.message || t('registerDialog.invalidCode'), t('registerDialog.verificationFailed'))
      // Clear the code input
      clearCodeInputs()
    }
  } catch (error: unknown) {
    verificationError.value = true
    showError(parseApiError(error, t('registerDialog.invalidCodeRetry')), t('registerDialog.verificationFailed'))
    // Clear the code input
    clearCodeInputs()
  } finally {
    isLoading.value = false
  }
}

const handleSubmit = async () => {
  // Validate password match
  if (formData.value.password !== formData.value.confirmPassword) {
    showError(t('registerDialog.passwordMismatch'), t('registerDialog.passwordMismatchTitle'))
    return
  }

  if (passwordError.value) {
    showError(passwordError.value, t('registerDialog.passwordError'))
    return
  }

  // Check email verification if required
  if (props.requireEmailVerification && !emailVerified.value) {
    showError(t('registerDialog.completeEmailVerification'))
    return
  }
  if (
    turnstileRequired.value &&
    currentTurnstileAction.value === 'register' &&
    !turnstileToken.value
  ) {
    showError(t('registerDialog.completeCaptchaFirst'))
    return
  }

  if (privacyPolicyEnabled.value && !privacyAccepted.value) {
    showError(t('registerDialog.acceptPrivacyFirst'))
    return
  }

  isLoading.value = true
  loadingText.value = t('registerDialog.registering')

  try {
    // 构建请求数据：邮箱可选
    const registerData: RegisterRequest = {
      username: formData.value.username,
      password: formData.value.password
    }
    // 只有当邮箱有值时才添加
    if (formData.value.email && formData.value.email.trim()) {
      registerData.email = formData.value.email
    }
    if (turnstileRequired.value && currentTurnstileAction.value === 'register') {
      registerData.turnstile_token = turnstileToken.value
    }
    if (inviteCode.value) {
      registerData.invite_code = inviteCode.value
    }
    if (privacyPolicyEnabled.value) {
      registerData.privacy_policy_accepted = privacyAccepted.value
      registerData.privacy_policy_version = privacyPolicyVersion.value
    }

    const response = await authApi.register(registerData)

    success(response.message || t('registerDialog.welcome'), t('registerDialog.registerSuccess'))

    emit('success')
    isOpen.value = false
  } catch (error: unknown) {
    resetTurnstile()
    showError(parseApiError(error, t('registerDialog.registerFailedRetry')), t('registerDialog.registerFailed'))
  } finally {
    isLoading.value = false
    resetTurnstile()
  }
}

const handleCancel = () => {
  isOpen.value = false
}

const handleSwitchToLogin = () => {
  emit('switchToLogin')
  isOpen.value = false
}
</script>
