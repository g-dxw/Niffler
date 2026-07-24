<template>
  <PageContainer>
    <PageHeader
      :title="t('upstreamOnboarding.title')"
      :description="t('upstreamOnboarding.description')"
      :icon="Server"
    >
      <template #actions>
        <Button
          variant="outline"
          class="admin-filter-action"
          :disabled="pageLoading"
          @click="refreshAll"
        >
          <RefreshCw
            class="mr-2 h-4 w-4"
            :class="{ 'animate-spin': pageLoading }"
          />
          {{ t('upstreamOnboarding.refresh') }}
        </Button>
        <Button
          class="admin-entry-action"
          @click="openServiceDialog"
        >
          <Plus class="mr-2 h-4 w-4" />
          {{ t('upstreamOnboarding.add') }}
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 space-y-5">
      <Card class="p-4">
        <div class="grid gap-3 md:grid-cols-3">
          <div
            v-for="step in onboardingSteps"
            :key="step.title"
            class="rounded-lg border border-border/70 bg-muted/20 px-4 py-3"
          >
            <div class="text-xs font-medium text-muted-foreground">
              {{ step.index }}
            </div>
            <div class="mt-1 text-sm font-semibold">
              {{ step.title }}
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ step.description }}
            </p>
          </div>
        </div>
      </Card>

      <Card class="overflow-hidden">
        <div class="grid min-h-[560px] xl:grid-cols-[360px_minmax(0,1fr)]">
          <section class="border-b border-border/70 p-4 xl:border-b-0 xl:border-r">
            <div class="flex items-center justify-between gap-3">
              <div>
                <h2 class="text-sm font-semibold">
                  {{ t('upstreamOnboarding.upstream') }}
                </h2>
                <p class="mt-1 text-xs text-muted-foreground">
                  {{ t('upstreamOnboarding.upstreamHint') }}
                </p>
              </div>
              <Badge variant="secondary">
                {{ services.length }}
              </Badge>
            </div>

            <div class="mt-3 flex gap-2">
              <Input
                v-model="serviceSearch"
                class="h-9"
                :placeholder="t('upstreamOnboarding.search')"
                @keyup.enter="loadServices"
              />
              <Button
                variant="outline"
                size="icon"
                class="admin-filter-action h-9 w-9 shrink-0"
                :disabled="serviceLoading"
                :title="t('upstreamOnboarding.searchAction')"
                @click="loadServices"
              >
                <Search class="h-4 w-4" />
              </Button>
            </div>

            <p
              v-if="serviceError"
              class="mt-3 rounded-md border border-destructive/20 bg-destructive/5 px-3 py-2 text-sm text-destructive"
            >
              {{ serviceError }}
            </p>

            <div
              v-if="serviceLoading && services.length === 0"
              class="flex items-center justify-center py-12 text-sm text-muted-foreground"
            >
              <Loader2 class="mr-2 h-5 w-5 animate-spin" />
              {{ t('upstreamOnboarding.loading') }}
            </div>

            <div
              v-else-if="services.length === 0"
              class="mt-4 rounded-lg border border-dashed border-border/70 p-4"
            >
              <p class="text-sm font-medium">
                {{ t('upstreamOnboarding.empty') }}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('upstreamOnboarding.emptyHint') }}
              </p>
            </div>

            <div
              v-else
              class="mt-3 space-y-2"
            >
              <button
                v-for="service in services"
                :key="service.id"
                type="button"
                class="admin-entry-row w-full rounded-lg border px-3 py-2 text-left transition-colors"
                :class="selectedServiceId === service.id ? 'border-primary/50 bg-primary/10' : 'border-border/70 hover:bg-muted/40'"
                @click="selectService(service.id)"
              >
                <div class="flex items-start justify-between gap-2">
                  <div class="min-w-0">
                    <div class="truncate text-sm font-medium">
                      {{ service.display_name }}
                    </div>
                    <div class="mt-1 truncate text-xs text-muted-foreground">
                      {{ serviceKindLabel(service.service_kind) }} · {{ service.default_api_format || '-' }}
                    </div>
                  </div>
                  <Badge :variant="service.is_active ? 'outline' : 'secondary'">
                    {{ service.is_active ? t('upstreamOnboarding.enabled') : t('upstreamOnboarding.disabled') }}
                  </Badge>
                </div>
              </button>
            </div>
          </section>

          <section class="p-4">
            <div
              v-if="!selectedService"
              class="flex min-h-[440px] flex-col justify-center rounded-lg border border-dashed border-border/70 p-6"
            >
              <p class="text-base font-semibold">
                {{ services.length === 0 ? t('upstreamOnboarding.waiting') : t('upstreamOnboarding.select') }}
              </p>
              <p class="mt-2 max-w-md text-sm text-muted-foreground">
                {{ services.length === 0 ? t('upstreamOnboarding.emptyDetail') : t('upstreamOnboarding.selectDetail') }}
              </p>
            </div>

            <div
              v-else
              class="space-y-5"
            >
              <div class="flex flex-col gap-3 border-b border-border/70 pb-4 lg:flex-row lg:items-start lg:justify-between">
                <div class="min-w-0">
                  <div class="flex flex-wrap items-center gap-2">
                    <h2 class="truncate text-lg font-semibold">
                      {{ selectedService.display_name }}
                    </h2>
                    <Badge :variant="selectedService.is_active ? 'outline' : 'secondary'">
                      {{ selectedService.is_active ? t('upstreamOnboarding.enabled') : t('upstreamOnboarding.disabled') }}
                    </Badge>
                    <Badge variant="secondary">
                      {{ serviceKindLabel(selectedService.service_kind) }}
                    </Badge>
                  </div>
                  <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
                    <span>{{ t('upstreamOnboarding.protocolValue', { value: selectedService.default_api_format || '-' }) }}</span>
                    <span>{{ t('upstreamOnboarding.costMultiplierValue', { value: formatMultiplier(selectedService.cost_multiplier) }) }}</span>
                    <span>{{ t('upstreamOnboarding.accountCount', { count: accounts.length }) }}</span>
                  </div>
                  <p
                    v-if="selectedService.base_url"
                    class="mt-2 truncate text-xs text-muted-foreground"
                  >
                    {{ selectedService.base_url }}
                  </p>
                </div>
                <Button
                  class="admin-entry-action h-9"
                  @click="accountDialogOpen = true"
                >
                  <Plus class="mr-2 h-4 w-4" />
                  {{ t('upstreamOnboarding.addAccount') }}
                </Button>
              </div>

              <div class="grid gap-5 xl:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
                <section class="rounded-lg border border-border/70">
                  <div class="flex items-center justify-between border-b border-border/70 px-4 py-3">
                    <div>
                      <h3 class="text-sm font-semibold">
                        {{ t('upstreamOnboarding.account') }}
                      </h3>
                    </div>
                    <Badge variant="secondary">
                      {{ accounts.length }}
                    </Badge>
                  </div>

                  <p
                    v-if="accountError"
                    class="border-b border-destructive/20 bg-destructive/5 px-4 py-3 text-sm text-destructive"
                  >
                    {{ accountError }}
                  </p>

                  <div
                    v-if="accountLoading && accounts.length === 0"
                    class="flex items-center justify-center py-12 text-sm text-muted-foreground"
                  >
                    <Loader2 class="mr-2 h-5 w-5 animate-spin" />
                    {{ t('upstreamOnboarding.loadingAccounts') }}
                  </div>

                  <div
                    v-else-if="accounts.length === 0"
                    class="p-4"
                  >
                    <div class="rounded-lg border border-dashed border-border/70 p-4">
                      <p class="text-sm font-medium">
                        {{ t('upstreamOnboarding.noAccounts') }}
                      </p>
                      <p class="mt-1 text-xs text-muted-foreground">
                        {{ t('upstreamOnboarding.accountHint') }}
                      </p>
                      <Button
                        class="admin-entry-action mt-3 h-8"
                        size="sm"
                        @click="accountDialogOpen = true"
                      >
                        {{ t('upstreamOnboarding.addAccount') }}
                      </Button>
                    </div>
                  </div>

                  <div
                    v-else
                    class="divide-y divide-border/70"
                  >
                    <div
                      v-for="account in accounts"
                      :key="account.id"
                      class="px-4 py-3"
                    >
                      <div class="flex items-start justify-between gap-3">
                        <div class="min-w-0">
                          <p class="truncate text-sm font-medium">
                            {{ account.display_name }}
                          </p>
                          <p class="mt-1 truncate text-xs text-muted-foreground">
                            {{ accountContactLabel(account) }}
                          </p>
                        </div>
                        <Badge :variant="account.status === 'available' ? 'outline' : 'secondary'">
                          {{ accountStatusLabel(account.status) }}
                        </Badge>
                      </div>
                      <div class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
                        <span>{{ authKindLabel(account.auth_kind) }}</span>
                        <span>{{ t('upstreamOnboarding.costMultiplierValue', { value: formatMultiplier(account.cost_multiplier) }) }}</span>
                        <span>{{ t('upstreamOnboarding.priorityValue', { value: account.priority }) }}</span>
                        <span>{{ t('upstreamOnboarding.testValue', { value: accountTestStatus(account) }) }}</span>
                      </div>
                      <p
                        v-if="account.cooldown_until_unix_ms"
                        class="mt-1 text-xs text-muted-foreground"
                      >
                        {{ t('upstreamOnboarding.cooldownUntil', { value: formatNifflerUnixMs(account.cooldown_until_unix_ms) }) }}
                      </p>
                      <p
                        v-if="account.last_test_error"
                        class="mt-2 line-clamp-2 text-xs text-destructive"
                      >
                        {{ account.last_test_error }}
                      </p>
                    </div>
                  </div>
                </section>

                <section class="rounded-lg border border-border/70">
                  <div class="flex flex-col gap-3 border-b border-border/70 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
                    <div>
                      <h3 class="text-sm font-semibold">
                        {{ t('upstreamOnboarding.capability') }}
                      </h3>
                      <p class="mt-1 text-xs text-muted-foreground">
                        {{ t('upstreamOnboarding.capabilityHint') }}
                      </p>
                    </div>
                    <div class="flex gap-2">
                      <Button
                        variant="outline"
                        class="admin-filter-action"
                        size="sm"
                        :disabled="serviceCapabilityLoading"
                        @click="checkServiceCapabilities"
                      >
                        {{ t('upstreamOnboarding.check') }}
                      </Button>
                      <Button
                        class="admin-entry-action"
                        size="sm"
                        :disabled="savingServiceCapabilities || serviceCapabilityLoading"
                        @click="submitServiceCapabilities"
                      >
                        {{ savingServiceCapabilities ? t('upstreamOnboarding.saving') : t('upstreamOnboarding.saveCapability') }}
                      </Button>
                    </div>
                  </div>

                  <p
                    v-if="serviceCapabilityError"
                    class="border-b border-destructive/20 bg-destructive/5 px-4 py-3 text-sm text-destructive"
                  >
                    {{ serviceCapabilityError }}
                  </p>

                  <div
                    v-if="serviceCapabilityLoading"
                    class="flex items-center justify-center py-12 text-sm text-muted-foreground"
                  >
                    <Loader2 class="mr-2 h-5 w-5 animate-spin" />
                    {{ t('upstreamOnboarding.loadingCapability') }}
                  </div>

                  <div
                    v-else
                    class="space-y-4 p-4"
                  >
                    <div class="grid gap-3 sm:grid-cols-[180px_minmax(0,1fr)]">
                      <div class="space-y-2">
                        <Label for="service-capability-protocol">{{ t('upstreamOnboarding.protocol') }}</Label>
                        <Select v-model="serviceCapabilityForm.protocol_kind">
                          <SelectTrigger id="service-capability-protocol">
                            <SelectValue :placeholder="t('upstreamOnboarding.selectProtocol')" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="openai">
                              OpenAI
                            </SelectItem>
                            <SelectItem value="anthropic">
                              Anthropic
                            </SelectItem>
                            <SelectItem value="gemini">
                              Gemini
                            </SelectItem>
                            <SelectItem value="codex">
                              Codex
                            </SelectItem>
                            <SelectItem value="custom">
                              {{ t('upstreamOnboarding.custom') }}
                            </SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div class="rounded-lg border border-border/70 bg-muted/20 px-3 py-2">
                        <div class="text-xs text-muted-foreground">
                          {{ t('upstreamOnboarding.active') }}
                        </div>
                        <div
                          v-if="selectedServiceCapabilityLabels.length > 0"
                          class="mt-2 flex flex-wrap gap-1.5"
                        >
                          <Badge
                            v-for="label in selectedServiceCapabilityLabels"
                            :key="label"
                            variant="outline"
                          >
                            {{ label }}
                          </Badge>
                        </div>
                        <p
                          v-else
                          class="mt-2 text-sm text-muted-foreground"
                        >
                          {{ t('upstreamOnboarding.inactive') }}
                        </p>
                      </div>
                    </div>

                    <div class="grid gap-2 sm:grid-cols-2">
                      <label
                        v-for="item in selectedServiceCapabilityOptions"
                        :key="item.key"
                        class="flex items-start gap-3 rounded-lg border border-border/70 p-3"
                      >
                        <Checkbox v-model:checked="serviceCapabilityForm.capabilities[item.key]" />
                        <span>
                          <span class="block text-sm font-medium">{{ item.label }}</span>
                          <span class="block text-xs text-muted-foreground">{{ item.description }}</span>
                        </span>
                      </label>
                    </div>

                    <p
                      v-if="serviceCapabilityIssues.length > 0"
                      class="text-xs text-destructive"
                    >
                      {{ serviceCapabilityIssues.join(' ') }}
                    </p>
                  </div>
                </section>
              </div>
            </div>
          </section>
        </div>
      </Card>
    </div>

    <Dialog
      v-model="serviceDialogOpen"
      size="2xl"
      :title="t('upstreamOnboarding.addTitle')"
      :description="t('upstreamOnboarding.addDescription')"
      :icon="Server"
    >
      <form
        class="space-y-5"
        @submit.prevent="submitService"
      >
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="service-name">{{ t('upstreamOnboarding.serviceName') }}</Label>
            <Input
              id="service-name"
              v-model="serviceForm.display_name"
              :placeholder="t('upstreamOnboarding.serviceNamePlaceholder')"
              required
            />
          </div>
          <div class="space-y-2">
            <Label for="service-template">{{ t('upstreamOnboarding.integrationType') }}</Label>
            <Select v-model="selectedServiceTemplateKey">
              <SelectTrigger id="service-template">
                <SelectValue :placeholder="t('upstreamOnboarding.selectType')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="template in nifflerServiceTemplates"
                  :key="template.key"
                  :value="template.key"
                >
                  {{ template.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="rounded-xl border border-border/70 bg-muted/30 p-4 sm:col-span-2">
            <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <p class="text-sm text-muted-foreground">
                {{ selectedServiceTemplate.description }}
              </p>
              <Badge variant="outline">
                {{ t('upstreamOnboarding.defaultAuth', { value: authKindLabel(selectedServiceTemplate.defaultAuthKind) }) }}
              </Badge>
            </div>
            <div class="mt-3 flex flex-wrap gap-2 text-xs text-muted-foreground">
              <span class="rounded-md bg-background px-2 py-1">{{ t('upstreamOnboarding.typeValue', { value: serviceKindLabel(serviceForm.service_kind) }) }}</span>
              <span class="rounded-md bg-background px-2 py-1">{{ t('upstreamOnboarding.protocolValue', { value: serviceForm.protocol_kind }) }}</span>
              <span class="rounded-md bg-background px-2 py-1">{{ t('upstreamOnboarding.formatValue', { value: serviceForm.default_api_format }) }}</span>
            </div>
          </div>

          <div class="space-y-2 sm:col-span-2">
            <Label for="base-url">Base URL</Label>
            <Input
              id="base-url"
              v-model="serviceForm.base_url"
              :placeholder="selectedServiceTemplate.baseUrlPlaceholder"
              :required="selectedServiceTemplate.baseUrlRequired"
            />
          </div>
          <div class="space-y-2">
            <Label for="cost-multiplier">{{ t('upstreamOnboarding.costMultiplier') }}</Label>
            <Input
              id="cost-multiplier"
              v-model.number="serviceForm.cost_multiplier"
              type="number"
              min="0"
              step="0.0001"
            />
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="service-active"
              v-model="serviceForm.is_active"
            />
            <Label for="service-active">{{ t('upstreamOnboarding.enableUpstream') }}</Label>
          </div>

          <details class="rounded-xl border border-border/70 p-4 sm:col-span-2">
            <summary class="cursor-pointer text-sm font-medium">
              {{ t('upstreamOnboarding.advancedFields') }}
            </summary>
            <div class="mt-4 grid gap-4 sm:grid-cols-3">
              <div class="space-y-2">
                <Label for="service-kind">{{ t('upstreamOnboarding.serviceType') }}</Label>
                <Input
                  id="service-kind"
                  v-model="serviceForm.service_kind"
                  :placeholder="t('upstreamOnboarding.serviceTypePlaceholder')"
                />
              </div>
              <div class="space-y-2">
                <Label for="protocol-kind">{{ t('upstreamOnboarding.protocol') }}</Label>
                <Select v-model="serviceForm.protocol_kind">
                  <SelectTrigger id="protocol-kind">
                    <SelectValue :placeholder="t('upstreamOnboarding.selectProtocol')" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="openai">
                      OpenAI
                    </SelectItem>
                    <SelectItem value="anthropic">
                      Anthropic
                    </SelectItem>
                    <SelectItem value="gemini">
                      Gemini
                    </SelectItem>
                    <SelectItem value="codex">
                      Codex
                    </SelectItem>
                    <SelectItem value="custom">
                      {{ t('upstreamOnboarding.custom') }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="space-y-2">
                <Label for="api-format">{{ t('upstreamOnboarding.defaultFormat') }}</Label>
                <Input
                  id="api-format"
                  v-model="serviceForm.default_api_format"
              :placeholder="t('upstreamOnboarding.defaultFormatPlaceholder')"
                />
              </div>
            </div>
          </details>
        </div>

        <div class="rounded-xl border border-border/70 p-4">
          <p class="text-sm font-medium">
            {{ t('upstreamOnboarding.defaultCapabilities') }}
          </p>
          <p class="mt-1 text-xs text-muted-foreground">
            {{ t('upstreamOnboarding.defaultCapabilitiesHint') }}
          </p>
          <div class="mt-3 grid gap-3 md:grid-cols-2">
            <label
              v-for="item in visibleCapabilityOptions"
              :key="item.key"
              class="flex min-w-0 items-start gap-3 rounded-lg border border-border/50 p-3"
            >
              <Checkbox v-model:checked="serviceCapabilities[item.key]" class="mt-0.5 shrink-0" />
              <span class="min-w-0">
                <span class="block break-words text-sm font-medium leading-5">{{ item.label }}</span>
                <span class="mt-1 block break-words text-xs leading-5 text-muted-foreground">{{ item.description }}</span>
              </span>
            </label>
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          class="admin-entry-action"
          type="submit"
          :disabled="savingService"
          @click="submitService"
        >
          {{ savingService ? t('upstreamOnboarding.saving') : t('upstreamOnboarding.saveUpstream') }}
        </Button>
        <Button
          class="admin-entry-action"
          type="button"
          variant="outline"
          :disabled="savingService"
          @click="serviceDialogOpen = false"
        >
          {{ t('upstreamOnboarding.cancel') }}
        </Button>
      </template>
    </Dialog>

    <Dialog
      v-model="accountDialogOpen"
      size="lg"
      :title="t('upstreamOnboarding.addAccountTitle')"
      :description="t('upstreamOnboarding.accountHint')"
      :icon="KeyRound"
    >
      <form
        class="space-y-4"
        @submit.prevent="submitAccount"
      >
        <div class="rounded-xl border border-border/70 bg-muted/25 p-4">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p class="text-sm font-medium">
                {{ accountAuthGuide.title }}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ accountAuthGuide.description }}
              </p>
            </div>
            <Badge variant="outline">
              {{ authKindLabel(accountForm.auth_kind) }}
            </Badge>
          </div>
        </div>

        <div class="space-y-2">
          <Label for="account-name">{{ t('upstreamOnboarding.accountName') }}</Label>
          <Input
            id="account-name"
            v-model="accountForm.display_name"
            :placeholder="accountAuthGuide.namePlaceholder"
            required
          />
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="account-email">{{ t('upstreamOnboarding.email') }}</Label>
            <Input
              id="account-email"
              v-model="accountForm.email"
              placeholder="name@example.com"
            />
          </div>
          <div class="space-y-2">
            <Label for="account-phone">{{ t('upstreamOnboarding.phone') }}</Label>
            <Input
              id="account-phone"
              v-model="accountForm.phone"
              :placeholder="t('upstreamOnboarding.optional')"
            />
          </div>
        </div>

        <p class="text-xs text-muted-foreground">
          {{ accountAuthGuide.contactHint }}
        </p>

        <div class="grid gap-4 sm:grid-cols-3">
          <div class="space-y-2">
            <Label for="account-auth">{{ t('upstreamOnboarding.authMethod') }}</Label>
            <Select v-model="accountForm.auth_kind">
              <SelectTrigger id="account-auth">
                <SelectValue :placeholder="t('upstreamOnboarding.selectMethod')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="oauth">
                  OAuth
                </SelectItem>
                <SelectItem value="api_key">
                  API Key
                </SelectItem>
                <SelectItem value="custom_header">
                  {{ t('upstreamOnboarding.customHeader') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-2">
            <Label for="account-cost">{{ t('upstreamOnboarding.costMultiplier') }}</Label>
            <Input
              id="account-cost"
              v-model.number="accountForm.cost_multiplier"
              type="number"
              min="0"
              step="0.0001"
            />
          </div>
          <div class="space-y-2">
            <Label for="account-priority">{{ t('upstreamOnboarding.priority') }}</Label>
            <Input
              id="account-priority"
              v-model.number="accountForm.priority"
              type="number"
              step="1"
            />
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          class="admin-entry-action"
          type="submit"
          :disabled="savingAccount || !selectedService"
          @click="submitAccount"
        >
          {{ savingAccount ? t('upstreamOnboarding.saving') : t('upstreamOnboarding.saveAccount') }}
        </Button>
        <Button
          class="admin-entry-action"
          type="button"
          variant="outline"
          :disabled="savingAccount"
          @click="accountDialogOpen = false"
        >
          {{ t('upstreamOnboarding.cancel') }}
        </Button>
      </template>
    </Dialog>
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import {
  KeyRound,
  Loader2,
  Plus,
  RefreshCw,
  Search,
  Server,
} from 'lucide-vue-next'
import { PageContainer, PageHeader } from '@/components/layout'
import {
  Badge,
  Button,
  Card,
  Checkbox,
  Dialog,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from '@/components/ui'
import {
  createNifflerUpstreamAccount,
  createNifflerUpstreamService,
  listNifflerUpstreamAccounts,
  listNifflerUpstreamServiceCapabilities,
  listNifflerUpstreamServices,
  updateNifflerUpstreamServiceCapabilities,
  type CreateNifflerUpstreamAccountPayload,
  type CreateNifflerUpstreamServicePayload,
  type NifflerAccountStatus,
  type NifflerProtocolKind,
  type NifflerUpstreamAccount,
  type NifflerUpstreamService,
  type NifflerUpstreamServiceCapability,
  type UpdateNifflerUpstreamServiceCapabilitiesPayload,
} from '@/api/niffler-core'
import { useToast } from '@/composables/useToast'
import { extractErrorMessage } from '@/utils/error'
import {
  DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY,
  buildNifflerServiceCapabilityForm,
  buildNifflerServiceFormFromTemplate,
  enabledCapabilityLabels,
  filterCapabilityOptionsForProtocol,
  getDefaultAuthKindForService,
  getNifflerServiceTemplate,
  getServiceKindLabel,
  nifflerServiceTemplates,
  validateNifflerServiceCapabilities,
  type NifflerServiceCapabilityForm,
  type NifflerServiceCapabilityKey,
  type NifflerServiceTemplateKey,
} from './niffler-upstream-service-templates'
import {
  formatNifflerAccountTestStatus,
  formatNifflerUnixMs,
  getNifflerAccountAuthGuide,
} from './niffler-upstream-account-ui'

const { success, error: showError } = useToast()

const onboardingSteps = computed(() => [
  {
    index: '01',
    title: t('upstreamOnboarding.stepChooseType'),
    description: t('upstreamOnboarding.stepChooseTypeHint'),
  },
  {
    index: '02',
    title: t('upstreamOnboarding.stepAddAccount'),
    description: t('upstreamOnboarding.stepAddAccountHint'),
  },
  {
    index: '03',
    title: t('upstreamOnboarding.stepSaveCapabilities'),
    description: t('upstreamOnboarding.stepSaveCapabilitiesHint'),
  },
])

const capabilityOptions = computed<Array<{
  key: NifflerServiceCapabilityKey
  label: string
  description: string
}>>(() => [
  { key: 'text', label: t('upstreamCapabilities.text'), description: t('upstreamCapabilities.textHint') },
  { key: 'streaming', label: t('upstreamCapabilities.streaming'), description: t('upstreamCapabilities.streamingHint') },
  { key: 'images_endpoint', label: t('upstreamCapabilities.images'), description: t('upstreamCapabilities.imagesHint') },
  { key: 'openai_responses_image_tool', label: t('upstreamCapabilities.imageTool'), description: t('upstreamCapabilities.imageToolHint') },
  { key: 'model_list', label: t('upstreamCapabilities.modelList'), description: t('upstreamCapabilities.modelListHint') },
  { key: 'model_test', label: t('upstreamCapabilities.modelTest'), description: t('upstreamCapabilities.modelTestHint') },
])

const services = ref<NifflerUpstreamService[]>([])
const loadedServiceCapabilities = ref<NifflerUpstreamServiceCapability[]>([])
const accounts = ref<NifflerUpstreamAccount[]>([])
const serviceLoading = ref(false)
const serviceCapabilityLoading = ref(false)
const accountLoading = ref(false)
const savingService = ref(false)
const savingServiceCapabilities = ref(false)
const savingAccount = ref(false)
const serviceError = ref('')
const serviceCapabilityError = ref('')
const accountError = ref('')
const serviceSearch = ref('')
const selectedServiceId = ref<string | null>(null)
const serviceDialogOpen = ref(false)
const accountDialogOpen = ref(false)
const selectedServiceTemplateKey = ref<NifflerServiceTemplateKey>(DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY)
let accountLoadSeq = 0
let serviceCapabilityLoadSeq = 0

const defaultServiceForm = (): CreateNifflerUpstreamServicePayload =>
  buildNifflerServiceFormFromTemplate(DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY)

const defaultAccountForm = (): CreateNifflerUpstreamAccountPayload => ({
  display_name: '',
  email: '',
  phone: '',
  auth_kind: 'oauth',
  cost_multiplier: 1,
  priority: 0,
})

const serviceForm = ref<CreateNifflerUpstreamServicePayload>(defaultServiceForm())
type ServiceCapabilityFlags = NonNullable<CreateNifflerUpstreamServicePayload['capabilities']>

const serviceCapabilities = computed<ServiceCapabilityFlags>({
  get() {
    if (!serviceForm.value.capabilities) {
      serviceForm.value.capabilities = {}
    }
    return serviceForm.value.capabilities
  },
  set(value) {
    serviceForm.value.capabilities = value
  },
})
const serviceCapabilityForm = ref<NifflerServiceCapabilityForm>(
  buildNifflerServiceCapabilityForm(null)
)
const accountForm = ref<CreateNifflerUpstreamAccountPayload>(defaultAccountForm())

const pageLoading = computed(() =>
  serviceLoading.value || serviceCapabilityLoading.value || accountLoading.value
)

const selectedService = computed(() =>
  services.value.find(service => service.id === selectedServiceId.value) ?? null
)

const selectedServiceTemplate = computed(() =>
  getNifflerServiceTemplate(selectedServiceTemplateKey.value)
)

const visibleCapabilityOptions = computed(() =>
  filterCapabilityOptionsForProtocol(
    capabilityOptions.value,
    (serviceForm.value.protocol_kind || selectedServiceTemplate.value.protocolKind) as NifflerProtocolKind
  )
)

const selectedServiceCapabilityOptions = computed(() =>
  filterCapabilityOptionsForProtocol(
    capabilityOptions.value,
    serviceCapabilityForm.value.protocol_kind
  )
)

const selectedServiceCapabilityLabels = computed(() =>
  enabledCapabilityLabels(capabilityOptions.value, serviceCapabilityForm.value.capabilities)
)

const serviceCapabilityIssues = computed(() =>
  validateNifflerServiceCapabilities(serviceCapabilityForm.value)
)

const accountAuthGuide = computed(() =>
  getNifflerAccountAuthGuide(accountForm.value.auth_kind)
)

watch(serviceDialogOpen, (open) => {
  if (!open) {
    selectedServiceTemplateKey.value = DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY
    serviceForm.value = defaultServiceForm()
  }
})

watch(selectedServiceTemplateKey, (templateKey) => {
  serviceForm.value = buildNifflerServiceFormFromTemplate(templateKey, serviceForm.value)
})

watch(
  () => serviceForm.value.protocol_kind,
  (protocolKind) => {
    clearHiddenCapabilities((protocolKind || selectedServiceTemplate.value.protocolKind) as NifflerProtocolKind)
  }
)

watch(
  () => serviceCapabilityForm.value.protocol_kind,
  (protocolKind) => {
    clearHiddenServiceCapabilities(protocolKind)
  }
)

watch(accountDialogOpen, (open) => {
  if (!open) {
    accountForm.value = defaultAccountForm()
    return
  }
  if (selectedService.value) {
    accountForm.value = {
      ...defaultAccountForm(),
      auth_kind: getDefaultAuthKindForService(selectedService.value),
    }
  }
})

function openServiceDialog() {
  selectedServiceTemplateKey.value = DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY
  serviceForm.value = defaultServiceForm()
  serviceDialogOpen.value = true
}

function clearHiddenCapabilities(protocolKind: NifflerProtocolKind) {
  const capabilities = serviceForm.value.capabilities ?? {}
  clearHiddenCapabilityValues(capabilities, protocolKind)
  serviceForm.value.capabilities = capabilities
}

function clearHiddenServiceCapabilities(protocolKind: NifflerProtocolKind) {
  clearHiddenCapabilityValues(serviceCapabilityForm.value.capabilities, protocolKind)
}

function clearHiddenCapabilityValues(
  capabilities: Partial<Record<NifflerServiceCapabilityKey, boolean>>,
  protocolKind: NifflerProtocolKind
) {
  const visibleKeys = new Set(
    filterCapabilityOptionsForProtocol(capabilityOptions.value, protocolKind).map(option => option.key)
  )
  for (const option of capabilityOptions.value) {
    if (!visibleKeys.has(option.key)) {
      capabilities[option.key] = false
    }
  }
}

async function refreshAll() {
  await loadServices()
  if (selectedServiceId.value) {
    await Promise.all([
      loadAccounts(selectedServiceId.value),
      loadServiceCapabilities(selectedServiceId.value),
    ])
  }
}

async function loadServices() {
  serviceLoading.value = true
  serviceError.value = ''
  try {
    const response = await listNifflerUpstreamServices({
      include_inactive: true,
      search: serviceSearch.value.trim() || undefined,
      limit: 100,
    })
    services.value = response.items
    if (!selectedServiceId.value && services.value.length > 0) {
      await selectService(services.value[0].id)
    } else if (selectedServiceId.value && !services.value.some(item => item.id === selectedServiceId.value)) {
      selectedServiceId.value = services.value[0]?.id ?? null
      accounts.value = []
      loadedServiceCapabilities.value = []
      if (selectedServiceId.value) {
        await selectService(selectedServiceId.value)
      }
    }
  } catch (err) {
    serviceError.value = extractErrorMessage(err, t('upstreamOnboarding.loadFailed'))
    showError(serviceError.value)
  } finally {
    serviceLoading.value = false
  }
}

async function selectService(serviceId: string) {
  selectedServiceId.value = serviceId
  await Promise.all([
    loadAccounts(serviceId),
    loadServiceCapabilities(serviceId),
  ])
}

async function loadAccounts(serviceId: string) {
  const seq = ++accountLoadSeq
  accountLoading.value = true
  accountError.value = ''
  try {
    const response = await listNifflerUpstreamAccounts(serviceId, { limit: 100 })
    if (seq !== accountLoadSeq) return
    accounts.value = response.items
  } catch (err) {
    if (seq !== accountLoadSeq) return
    accountError.value = extractErrorMessage(err, t('upstreamOnboarding.loadAccountsFailed'))
    showError(accountError.value)
  } finally {
    if (seq === accountLoadSeq) {
      accountLoading.value = false
    }
  }
}

async function loadServiceCapabilities(serviceId: string) {
  const seq = ++serviceCapabilityLoadSeq
  serviceCapabilityLoading.value = true
  serviceCapabilityError.value = ''
  const service = services.value.find(item => item.id === serviceId) ?? selectedService.value
  serviceCapabilityForm.value = buildNifflerServiceCapabilityForm(service, [])
  try {
    const response = await listNifflerUpstreamServiceCapabilities(serviceId)
    if (seq !== serviceCapabilityLoadSeq) return
    loadedServiceCapabilities.value = response.items
    serviceCapabilityForm.value = buildNifflerServiceCapabilityForm(service, response.items)
  } catch (err) {
    if (seq !== serviceCapabilityLoadSeq) return
    serviceCapabilityError.value = extractErrorMessage(err, t('upstreamOnboarding.loadCapabilitiesFailed'))
    showError(serviceCapabilityError.value)
  } finally {
    if (seq === serviceCapabilityLoadSeq) {
      serviceCapabilityLoading.value = false
    }
  }
}

async function submitService() {
  const payload = normalizeServicePayload(serviceForm.value)
  if (!payload) return

  savingService.value = true
  try {
    const created = await createNifflerUpstreamService(payload)
    success(t('upstreamOnboarding.saved'))
    serviceDialogOpen.value = false
    await loadServices()
    await selectService(created.id)
  } catch (err) {
    showError(extractErrorMessage(err, t('upstreamOnboarding.createFailed')))
  } finally {
    savingService.value = false
  }
}

async function submitServiceCapabilities() {
  if (!selectedServiceId.value) return
  if (serviceCapabilityIssues.value.length > 0) {
    showError(serviceCapabilityIssues.value.join(' '))
    return
  }

  const payload: UpdateNifflerUpstreamServiceCapabilitiesPayload = {
    protocol_kind: serviceCapabilityForm.value.protocol_kind,
    capabilities: normalizeCapabilityPayload(serviceCapabilityForm.value.capabilities),
  }

  savingServiceCapabilities.value = true
  try {
    const response = await updateNifflerUpstreamServiceCapabilities(selectedServiceId.value, payload)
    loadedServiceCapabilities.value = response.items
    serviceCapabilityForm.value = buildNifflerServiceCapabilityForm(selectedService.value, response.items)
    success(t('upstreamOnboarding.capabilitiesSaved'))
  } catch (err) {
    showError(extractErrorMessage(err, t('upstreamOnboarding.saveCapabilitiesFailed')))
  } finally {
    savingServiceCapabilities.value = false
  }
}

function checkServiceCapabilities() {
  if (serviceCapabilityIssues.value.length > 0) {
    showError(serviceCapabilityIssues.value.join(' '))
    return
  }
  success(t('upstreamOnboarding.checkPassed'))
}

async function submitAccount() {
  if (!selectedServiceId.value) return
  const payload = normalizeAccountPayload(accountForm.value)
  if (!payload) return

  savingAccount.value = true
  try {
    await createNifflerUpstreamAccount(selectedServiceId.value, payload)
    success(t('upstreamOnboarding.accountSaved'))
    accountDialogOpen.value = false
    await loadAccounts(selectedServiceId.value)
  } catch (err) {
    showError(extractErrorMessage(err, t('upstreamOnboarding.addAccountFailed')))
  } finally {
    savingAccount.value = false
  }
}

function normalizeServicePayload(
  form: CreateNifflerUpstreamServicePayload
): CreateNifflerUpstreamServicePayload | null {
  const displayName = form.display_name.trim()
  if (!displayName) {
    showError(t('upstreamOnboarding.serviceNameRequired'))
    return null
  }

  const costMultiplier = Number(form.cost_multiplier ?? 1)
  if (!Number.isFinite(costMultiplier) || costMultiplier < 0) {
    showError(t('upstreamOnboarding.multiplierNonNegative'))
    return null
  }

  const protocolKind = (form.protocol_kind || 'openai') as NifflerProtocolKind
  return {
    display_name: displayName,
    service_kind: form.service_kind.trim() || 'custom',
    protocol_kind: protocolKind,
    default_api_format: emptyToNull(form.default_api_format),
    base_url: emptyToNull(form.base_url),
    cost_multiplier: costMultiplier,
    is_active: form.is_active ?? true,
    capabilities: normalizeCapabilityPayload(form.capabilities ?? {}),
  }
}

function normalizeCapabilityPayload(
  capabilities: Partial<Record<NifflerServiceCapabilityKey, boolean>>
): UpdateNifflerUpstreamServiceCapabilitiesPayload['capabilities'] {
  return {
    text: Boolean(capabilities.text),
    streaming: Boolean(capabilities.streaming),
    images_endpoint: Boolean(capabilities.images_endpoint),
    openai_responses_image_tool: Boolean(capabilities.openai_responses_image_tool),
    model_list: Boolean(capabilities.model_list),
    model_test: Boolean(capabilities.model_test),
  }
}

function normalizeAccountPayload(
  form: CreateNifflerUpstreamAccountPayload
): CreateNifflerUpstreamAccountPayload | null {
  const displayName = form.display_name.trim()
  if (!displayName) {
    showError(t('upstreamOnboarding.accountNameRequired'))
    return null
  }

  const costMultiplier = Number(form.cost_multiplier ?? 1)
  if (!Number.isFinite(costMultiplier) || costMultiplier < 0) {
    showError(t('upstreamOnboarding.multiplierNonNegative'))
    return null
  }

  const priority = Number(form.priority ?? 0)
  if (!Number.isFinite(priority)) {
    showError(t('upstreamOnboarding.priorityNumber'))
    return null
  }

  return {
    display_name: displayName,
    email: emptyToNull(form.email),
    phone: emptyToNull(form.phone),
    auth_kind: form.auth_kind,
    cost_multiplier: costMultiplier,
    priority,
  }
}

function emptyToNull(value?: string | null): string | null {
  const normalized = value?.trim() ?? ''
  return normalized ? normalized : null
}

function formatMultiplier(value: number): string {
  return `${Number(value || 0).toFixed(4).replace(/\.?0+$/, '')}x`
}

function accountContactLabel(account: NifflerUpstreamAccount): string {
  const contacts = [account.email, account.phone].filter(Boolean)
  return contacts.length > 0 ? contacts.join(' / ') : t('upstreamOnboarding.noContact')
}

function serviceKindLabel(value: string): string {
  return getServiceKindLabel(value)
}

function authKindLabel(value: string): string {
  const labels: Record<string, string> = {
    api_key: 'API Key',
    oauth: 'OAuth',
    custom_header: t('upstreamOnboarding.customHeader'),
  }
  return labels[value] ?? value
}

function accountStatusLabel(status: NifflerAccountStatus): string {
  const labels: Record<NifflerAccountStatus, string> = {
    available: t('upstreamOnboarding.available'),
    disabled: t('upstreamOnboarding.disabled'),
    invalid: t('upstreamOnboarding.invalid'),
    quota_exhausted: t('upstreamOnboarding.quotaExhausted'),
    cooling_down: t('upstreamOnboarding.coolingDown'),
  }
  return labels[status] ?? status
}

function accountTestStatus(account: NifflerUpstreamAccount): string {
  return formatNifflerAccountTestStatus(account)
}

onMounted(() => {
  void loadServices()
})
</script>
