<template>
  <PageContainer>
    <PageHeader
      :title="t('admin.ldap.title')"
      :description="t('admin.ldap.description')"
    />

    <div class="mt-6 space-y-6">
      <CardSection
        :title="t('admin.ldap.serverConfig')"
        :description="t('admin.ldap.serverHint')"
      >
        <template #actions>
          <div class="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              :disabled="testLoading"
              @click="handleTestConnection"
            >
              {{ testLoading ? t('admin.ldap.testing') : t('admin.ldap.test') }}
            </Button>
            <Button
              size="sm"
              :disabled="saveLoading"
              @click="handleSave"
            >
              {{ saveLoading ? t('admin.ldap.saving') : t('admin.ldap.save') }}
            </Button>
          </div>
        </template>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <Label
              for="server-url"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.serverUrl') }}
            </Label>
            <Input
              id="server-url"
              v-model="ldapConfig.server_url"
              type="text"
              placeholder="ldap://ldap.example.com:389"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('admin.ldap.serverUrlHint') }}
            </p>
          </div>

          <div>
            <Label
              for="bind-dn"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.bindDn') }}
            </Label>
            <Input
              id="bind-dn"
              v-model="ldapConfig.bind_dn"
              type="text"
              placeholder="cn=admin,dc=example,dc=com"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('admin.ldap.bindDnHint') }}
            </p>
          </div>

          <div>
            <Label
              for="bind-password"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.bindPassword') }}
            </Label>
            <div class="mt-1">
              <Input
                id="bind-password"
                v-model="ldapConfig.bind_password"
                masked
                :placeholder="hasPassword ? t('admin.ldap.passwordSet') : t('admin.ldap.passwordInput')"
                autocomplete="new-password"
              />
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('admin.ldap.bindPasswordHint') }}
            </p>
          </div>

          <div>
            <Label
              for="base-dn"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.baseDn') }}
            </Label>
            <Input
              id="base-dn"
              v-model="ldapConfig.base_dn"
              type="text"
              placeholder="ou=users,dc=example,dc=com"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('admin.ldap.baseDnHint') }}
            </p>
          </div>

          <div>
            <Label
              for="user-search-filter"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.searchFilter') }}
            </Label>
            <Input
              id="user-search-filter"
              v-model="ldapConfig.user_search_filter"
              type="text"
              placeholder="(uid={username})"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('admin.ldap.searchFilterHint') }}
            </p>
          </div>

          <div>
            <Label
              for="username-attr"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.usernameAttr') }}
            </Label>
            <Input
              id="username-attr"
              v-model="ldapConfig.username_attr"
              type="text"
              placeholder="uid"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('admin.ldap.usernameAttrHint') }}
            </p>
          </div>

          <div>
            <Label
              for="email-attr"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.emailAttr') }}
            </Label>
            <Input
              id="email-attr"
              v-model="ldapConfig.email_attr"
              type="text"
              placeholder="mail"
              class="mt-1"
            />
          </div>

          <div>
            <Label
              for="display-name-attr"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.displayNameAttr') }}
            </Label>
            <Input
              id="display-name-attr"
              v-model="ldapConfig.display_name_attr"
              type="text"
              placeholder="cn"
              class="mt-1"
            />
          </div>

          <div>
            <Label
              for="connect-timeout"
              class="block text-sm font-medium"
            >
              {{ t('admin.ldap.timeout') }}
            </Label>
            <Input
              id="connect-timeout"
              v-model.number="ldapConfig.connect_timeout"
              type="number"
              min="1"
              max="60"
              placeholder="10"
              class="mt-1"
            />
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('admin.ldap.timeoutHint') }}
            </p>
          </div>
        </div>

        <div class="mt-6 space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <Label class="text-sm font-medium">{{ t('admin.ldap.starttls') }}</Label>
              <p class="text-xs text-muted-foreground">
                {{ t('admin.ldap.starttlsHint') }}
              </p>
            </div>
            <Switch v-model="ldapConfig.use_starttls" />
          </div>

          <div class="flex items-center justify-between">
            <div>
              <Label class="text-sm font-medium">{{ t('admin.ldap.enabled') }}</Label>
              <p class="text-xs text-muted-foreground">
                {{ t('admin.ldap.enabledHint') }}
              </p>
            </div>
            <Switch v-model="ldapConfig.is_enabled" />
          </div>

          <div class="flex items-center justify-between">
            <div>
              <Label class="text-sm font-medium">{{ t('admin.ldap.exclusive') }}</Label>
              <p class="text-xs text-muted-foreground">
                {{ t('admin.ldap.exclusiveHint') }}
              </p>
            </div>
            <Switch v-model="ldapConfig.is_exclusive" />
          </div>
        </div>
      </CardSection>
    </div>
  </PageContainer>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { PageContainer, PageHeader, CardSection } from '@/components/layout'
import { Button, Input, Label, Switch } from '@/components/ui'
import { useToast } from '@/composables/useToast'
import { adminApi, type LdapConfigUpdateRequest } from '@/api/admin'
import { log } from '@/utils/logger'

const { t } = useI18n()
const { success, error } = useToast()

const loading = ref(false)
const saveLoading = ref(false)
const testLoading = ref(false)
const hasPassword = ref(false)

const ldapConfig = ref({
  server_url: '',
  bind_dn: '',
  bind_password: '',
  base_dn: '',
  user_search_filter: '(uid={username})',
  username_attr: 'uid',
  email_attr: 'mail',
  display_name_attr: 'cn',
  is_enabled: false,
  is_exclusive: false,
  use_starttls: false,
  connect_timeout: 10,
})

onMounted(async () => {
  await loadConfig()
})

async function loadConfig() {
  loading.value = true
  try {
    const response = await adminApi.getLdapConfig()
    ldapConfig.value = {
      server_url: response.server_url || '',
      bind_dn: response.bind_dn || '',
      bind_password: '',
      base_dn: response.base_dn || '',
      user_search_filter: response.user_search_filter || '(uid={username})',
      username_attr: response.username_attr || 'uid',
      email_attr: response.email_attr || 'mail',
      display_name_attr: response.display_name_attr || 'cn',
      is_enabled: response.is_enabled || false,
      is_exclusive: response.is_exclusive || false,
      use_starttls: response.use_starttls || false,
      connect_timeout: response.connect_timeout || 10,
    }
    hasPassword.value = !!response.has_bind_password
  } catch (err) {
    error(t('admin.ldap.loadFailed'))
    log.error('Failed to load LDAP configuration', err)
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  saveLoading.value = true
  try {
    const payload: LdapConfigUpdateRequest = {
      server_url: ldapConfig.value.server_url,
      bind_dn: ldapConfig.value.bind_dn,
      base_dn: ldapConfig.value.base_dn,
      user_search_filter: ldapConfig.value.user_search_filter,
      username_attr: ldapConfig.value.username_attr,
      email_attr: ldapConfig.value.email_attr,
      display_name_attr: ldapConfig.value.display_name_attr,
      is_enabled: ldapConfig.value.is_enabled,
      is_exclusive: ldapConfig.value.is_exclusive,
      use_starttls: ldapConfig.value.use_starttls,
      connect_timeout: ldapConfig.value.connect_timeout,
    }

    // 只有输入了新密码才更新密码
    if (ldapConfig.value.bind_password) {
      payload.bind_password = ldapConfig.value.bind_password
    }

    await adminApi.updateLdapConfig(payload)
    success(t('admin.ldap.saved'))

    if (ldapConfig.value.bind_password) {
      hasPassword.value = true
    }
    ldapConfig.value.bind_password = ''
  } catch (err) {
    error(t('admin.ldap.saveFailed'))
    log.error('Failed to save LDAP configuration', err)
  } finally {
    saveLoading.value = false
  }
}

async function handleTestConnection() {
  testLoading.value = true
  try {
    const payload: LdapConfigUpdateRequest = {
      server_url: ldapConfig.value.server_url,
      bind_dn: ldapConfig.value.bind_dn,
      base_dn: ldapConfig.value.base_dn,
      user_search_filter: ldapConfig.value.user_search_filter,
      username_attr: ldapConfig.value.username_attr,
      email_attr: ldapConfig.value.email_attr,
      display_name_attr: ldapConfig.value.display_name_attr,
      is_enabled: ldapConfig.value.is_enabled,
      is_exclusive: ldapConfig.value.is_exclusive,
      use_starttls: ldapConfig.value.use_starttls,
      connect_timeout: ldapConfig.value.connect_timeout,
      ...(ldapConfig.value.bind_password && { bind_password: ldapConfig.value.bind_password }),
    }
    const response = await adminApi.testLdapConnection(payload)
    if (response.success) {
      success(t('admin.ldap.testPassed'))
    } else {
      error(t('admin.ldap.testFailedDetail', { message: response.message }))
    }
  } catch (err) {
    error(t('admin.ldap.testFailed'))
    log.error('LDAP connection test failed', err)
  } finally {
    testLoading.value = false
  }
}
</script>
