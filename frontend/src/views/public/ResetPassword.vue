<template>
  <main class="min-h-screen bg-background text-foreground">
    <div class="mx-auto flex min-h-screen w-full max-w-md flex-col justify-center px-6 py-10">
      <div class="mb-8 flex flex-col items-center text-center">
        <img
          src="/aether_adaptive.svg"
          :alt="siteName"
          class="mb-4 h-14 w-14"
        >
        <h1 class="text-2xl font-semibold">
          {{ t('resetPassword.title') }}
        </h1>
      </div>

      <form
        v-if="hasToken && !resetDone"
        class="space-y-4 rounded-xl border border-border bg-card p-6 shadow-sm"
        @submit.prevent="handleResetPassword"
      >
        <div class="space-y-1.5">
          <Label
            for="new-password"
            class="text-sm"
          >
            {{ t('resetPassword.newPassword') }}
          </Label>
          <Input
            id="new-password"
            v-model="password"
            type="password"
            required
            autocomplete="new-password"
            :placeholder="t('resetPassword.newPasswordPlaceholder')"
          />
        </div>
        <div class="space-y-1.5">
          <Label
            for="confirm-password"
            class="text-sm"
          >
            {{ t('resetPassword.confirmPassword') }}
          </Label>
          <Input
            id="confirm-password"
            v-model="confirmPassword"
            type="password"
            required
            autocomplete="new-password"
            :placeholder="t('resetPassword.confirmPasswordPlaceholder')"
          />
        </div>
        <p
          v-if="errorMessage"
          class="rounded-lg border border-destructive/20 bg-destructive/5 px-3 py-2 text-sm text-destructive"
        >
          {{ errorMessage }}
        </p>
        <Button
          type="submit"
          :disabled="submitting"
          class="h-11 w-full"
        >
          {{ submitting ? t('resetPassword.submitting') : t('resetPassword.title') }}
        </Button>
      </form>

      <div
        v-else
        class="rounded-xl border border-border bg-card p-6 text-center shadow-sm"
      >
        <p class="text-sm text-muted-foreground">
          {{ resetDone ? successMessage : t('resetPassword.invalidMissing') }}
        </p>
      </div>

      <button
        type="button"
        class="mt-5 text-sm text-muted-foreground transition-colors hover:text-foreground"
        @click="goHome"
      >
        {{ t('resetPassword.backHome') }}
      </button>
    </div>
  </main>
  <PublicFooter />
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { authApi } from '@/api/auth'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import PublicFooter from '@/components/common/PublicFooter.vue'
import { useSiteInfo } from '@/composables/useSiteInfo'
import { getErrorMessage } from '@/types/api-error'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { siteName } = useSiteInfo()

const token = computed(() => {
  const raw = route.query.token
  return typeof raw === 'string' ? raw.trim() : ''
})
const hasToken = computed(() => token.value.length > 0)

const password = ref('')
const confirmPassword = ref('')
const submitting = ref(false)
const resetDone = ref(false)
const errorMessage = ref('')
const successMessage = ref(t('resetPassword.success'))

async function handleResetPassword() {
  errorMessage.value = ''
  if (!hasToken.value) {
    errorMessage.value = t('resetPassword.invalidExpired')
    return
  }
  if (!password.value || !confirmPassword.value) {
    errorMessage.value = t('resetPassword.enterPassword')
    return
  }
  if (password.value !== confirmPassword.value) {
    errorMessage.value = t('resetPassword.mismatch')
    return
  }

  submitting.value = true
  try {
    const response = await authApi.resetPassword(token.value, password.value)
    successMessage.value = response.message || successMessage.value
    resetDone.value = true
  } catch (error) {
    errorMessage.value = getErrorMessage(error, t('resetPassword.failed'))
  } finally {
    submitting.value = false
  }
}

function goHome() {
  router.push('/')
}
</script>
