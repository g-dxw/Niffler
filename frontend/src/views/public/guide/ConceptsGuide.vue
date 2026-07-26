<script setup lang="ts">
import { BookOpen } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <div class="space-y-12">
    <!-- Hero 区域 -->
    <div class="space-y-4">
      <div class="inline-flex items-center gap-1.5 rounded-full bg-[#cc785c]/10 dark:bg-[#cc785c]/20 border border-[#cc785c]/20 dark:border-[#cc785c]/40 px-3 py-1 text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
        <BookOpen class="h-3 w-3" />
        {{ t('conceptsGuide.badge') }}
      </div>
      <h1 class="text-3xl font-bold text-[#262624] dark:text-[#f1ead8]">
        {{ t('conceptsGuide.title') }}
      </h1>
      <p class="text-base text-[#666663] dark:text-[#a3a094] max-w-2xl">
        {{ t('conceptsGuide.description') }}
      </p>
    </div>

    <!-- 1. 创建统一模型 -->
    <section
      id="create-model"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.createModelTitle') }}</h2>
      <div class="space-y-4 mt-4">
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8]">
            {{ t('conceptsGuide.modelPickerTitle') }}
          </h4>
          <ul class="list-disc pl-5 mt-2 space-y-1 text-sm text-[#666663] dark:text-[#a3a094]">
            <li>{{ t('conceptsGuide.modelSource') }}</li>
            <li>{{ t('conceptsGuide.modelSearchHint') }}</li>
          </ul>
        </div>
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8]">
            {{ t('conceptsGuide.modelConfigTitle') }}
          </h4>
          <ul class="list-disc pl-5 mt-2 space-y-1 text-sm text-[#666663] dark:text-[#a3a094]">
            <li>{{ t('conceptsGuide.manualModel') }}</li>
            <li>{{ t('conceptsGuide.modelCapabilities') }}</li>
            <li>
              {{ t('conceptsGuide.pricingConfig') }}
              <ul class="list-[circle] pl-5 mt-1 space-y-1">
                <li>{{ t('conceptsGuide.pricingFormula') }}</li>
                <li>{{ t('conceptsGuide.tierPricing') }}</li>
              </ul>
            </li>
          </ul>
        </div>
        <img
          loading="lazy"
          src="/guide/concepts-create-model.webp"
          :alt="t('conceptsGuide.createModelTitle')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
        >
      </div>
    </section>

    <!-- 2. 添加提供商 -->
    <section
      id="add-provider"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.addProviderTitle') }}</h2>
      <div class="space-y-4 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <ul class="list-decimal pl-5 space-y-2">
          <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.providerType') }}：</strong>{{ t('conceptsGuide.providerTypeText') }}</li>
          <li>
            <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.billingType') }}：</strong>
            <ul class="list-disc pl-5 mt-1 space-y-1">
              <li>{{ t('conceptsGuide.payg') }}</li>
              <li>{{ t('conceptsGuide.periodQuota') }}</li>
              <li>{{ t('conceptsGuide.freeTier') }}</li>
            </ul>
          </li>
          <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.maxRetries') }}：</strong>{{ t('conceptsGuide.maxRetriesText') }}</li>
          <li>
            <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.timeout') }}：</strong>
            <ul class="list-disc pl-5 mt-1 space-y-1">
              <li>{{ t('conceptsGuide.streamTimeout') }}</li>
              <li>{{ t('conceptsGuide.nonStreamTimeout') }}</li>
            </ul>
          </li>
          <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.keepPriority') }}：</strong>{{ t('conceptsGuide.keepPriorityText') }}</li>
        </ul>
        <img
          loading="lazy"
          src="/guide/concepts-add-provider.webp"
          :alt="t('conceptsGuide.addProviderTitle')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm mt-4 w-full"
        >
      </div>
    </section>

    <!-- 3. 添加端点 -->
    <section
      id="add-endpoint"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.addEndpointTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('conceptsGuide.endpointIntro') }}
      </p>
      <ul class="list-decimal pl-5 space-y-2 text-[#666663] dark:text-[#a3a094] text-sm">
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.chooseFormat') }}：</strong>{{ t('conceptsGuide.chooseFormatText') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.customBasePath') }}：</strong>{{ t('conceptsGuide.customBasePathText') }}</li>
      </ul>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-add-endpoint-1.webp"
          :alt="t('conceptsGuide.chooseFormat')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <img
          loading="lazy"
          src="/guide/concepts-add-endpoint-2.webp"
          :alt="t('conceptsGuide.customBasePath')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
      </div>
    </section>

    <!-- 4. 添加密钥 -->
    <section
      id="add-key"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.addKeyTitle') }}</h2>
      <ul class="list-decimal pl-5 space-y-3 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.authType') }}：</strong>
          <ul class="list-disc pl-5 mt-1">
            <li>{{ t('conceptsGuide.apiKeyAuth') }}</li>
            <li>{{ t('conceptsGuide.vertexAuth') }}</li>
          </ul>
        </li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.supportedFormats') }}：</strong>{{ t('conceptsGuide.supportedFormatsText') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.priority') }}：</strong>{{ t('conceptsGuide.priorityText') }}</li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.rpmLimit') }}：</strong>{{ t('conceptsGuide.rpmText') }}
          <ul class="list-disc pl-5 mt-1">
            <li>{{ t('conceptsGuide.fixedValue') }}</li>
            <li>{{ t('conceptsGuide.adaptiveValue') }}</li>
          </ul>
        </li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.cacheTtl') }}：</strong>
          <ul class="list-disc pl-5 mt-1">
            <li>{{ t('conceptsGuide.cacheZero') }}</li>
            <li>{{ t('conceptsGuide.cacheMinutes') }}</li>
          </ul>
        </li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.accountStatus') }}：</strong><br>
          {{ t('conceptsGuide.accountStatusText') }}
        </li>
        <li>
          <strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('conceptsGuide.autoModels') }}：</strong><br>
          {{ t('conceptsGuide.autoModelsText') }}
        </li>
      </ul>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-add-key-1.webp"
          :alt="t('conceptsGuide.authType')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <img
          loading="lazy"
          src="/guide/concepts-add-key-2.webp"
          :alt="t('conceptsGuide.priority')"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
      </div>
    </section>

    <!-- 5. 模型权限 -->
    <section
      id="model-permission"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.modelPermissionTitle') }}</h2>
      <ul class="list-decimal pl-5 space-y-2 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <li>{{ t('conceptsGuide.permissionAutoFetch') }}</li>
        <li>{{ t('conceptsGuide.permissionManual') }}</li>
        <li>{{ t('conceptsGuide.permissionEmpty') }}</li>
        <li>{{ t('conceptsGuide.permissionMapping') }}</li>
      </ul>
      <div class="grid grid-cols-1 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-model-perms-1.webp"
          :alt="`${t('conceptsGuide.modelPermissionTitle')} 1`"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <img
            loading="lazy"
            src="/guide/concepts-model-perms-2.webp"
            :alt="`${t('conceptsGuide.modelPermissionTitle')} 2`"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
          <img
            loading="lazy"
            src="/guide/concepts-model-perms-3.webp"
            :alt="`${t('conceptsGuide.modelPermissionTitle')} 3`"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
        </div>
      </div>
    </section>

    <!-- 6. 关联模型 -->
    <section
      id="link-model"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.linkModelTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('conceptsGuide.linkModelText') }}
      </p>
      <div class="grid grid-cols-1 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-link-model-1.webp"
          :alt="`${t('conceptsGuide.linkModelTitle')} 1`"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <img
            loading="lazy"
            src="/guide/concepts-link-model-2.webp"
            :alt="`${t('conceptsGuide.linkModelTitle')} 2`"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
          <img
            loading="lazy"
            src="/guide/concepts-link-model-3.webp"
            :alt="`${t('conceptsGuide.linkModelTitle')} 3`"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
        </div>
      </div>
    </section>

    <!-- 7. 模型映射 -->
    <section
      id="model-mapping"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.modelMappingTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('conceptsGuide.modelMappingText') }}
      </p>
      <div class="bg-[#f5f5f0]/80 dark:bg-[rgba(227,224,211,0.05)] p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] text-sm mb-6">
        <p class="mb-2">
          <span class="text-[#cc785c] font-medium mr-2">{{ t('conceptsGuide.nameCorrection') }}:</span>{{ t('conceptsGuide.standardNameIs') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-opus-4-6</code> ，{{ t('conceptsGuide.providerNamed') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-opus-4-6-last</code>。
        </p>
        <p><span class="text-[#cc785c] font-medium mr-2">{{ t('conceptsGuide.upgradeExample') }}:</span>{{ t('conceptsGuide.standardNameIs') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-opus-4-6</code> ，{{ t('conceptsGuide.requestProviderUses') }} <code class="bg-black/5 dark:bg-white/10 px-1 py-0.5 rounded text-xs">claude-sonnet-4-6</code>。</p>
      </div>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
        <img
          loading="lazy"
          src="/guide/concepts-model-mapping-1.webp"
          :alt="`${t('conceptsGuide.modelMappingTitle')} 1`"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
        <img
          loading="lazy"
          src="/guide/concepts-model-mapping-2.webp"
          :alt="`${t('conceptsGuide.modelMappingTitle')} 2`"
          class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
        >
      </div>
    </section>

    <!-- 8. 反向代理 -->
    <section
      id="reverse-proxy"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('conceptsGuide.reverseProxyTitle') }}</h2>
      <div class="space-y-6 mt-6">
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8] mb-3">
            {{ t('conceptsGuide.codexProxy') }}
          </h4>
          <img
            loading="lazy"
            src="/guide/concepts-reverse-proxy-codex.webp"
                :alt="t('conceptsGuide.codexProxyAlt')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full max-w-2xl"
          >
        </div>
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8] mb-3">
            {{ t('conceptsGuide.kiroProxy') }}
          </h4>
          <img
            loading="lazy"
            src="/guide/concepts-reverse-proxy-kiro.webp"
                :alt="t('conceptsGuide.kiroProxyAlt')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full max-w-2xl"
          >
        </div>
        <div>
          <h4 class="font-medium text-[#262624] dark:text-[#f1ead8] mb-3">
            {{ t('conceptsGuide.antigravityProxy') }}
          </h4>
          <img
            loading="lazy"
            src="/guide/concepts-reverse-proxy-antigravity.webp"
                :alt="t('conceptsGuide.antigravityProxyAlt')"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full max-w-2xl"
          >
        </div>
      </div>
    </section>

    <!-- 9. 优先级管理 -->
    <section
      id="priority-management"
      class="scroll-mt-24 lg:scroll-mt-20 pb-8"
    >
      <h2>{{ t('conceptsGuide.priorityManagementTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('conceptsGuide.priorityManagementText') }}
      </p>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        <div class="space-y-4">
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('conceptsGuide.providerFirst') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('conceptsGuide.providerFirstText') }}
            </p>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('conceptsGuide.keyFirst') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('conceptsGuide.keyFirstText') }}
            </p>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('conceptsGuide.cacheAffinity') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('conceptsGuide.cacheAffinityText') }}
            </p>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('conceptsGuide.loadBalance') }}
            </h4>
            <ul class="list-disc pl-5 mt-1 text-sm text-[#666663] dark:text-[#a3a094] space-y-1">
              <li>{{ t('conceptsGuide.loadBalanceItem1') }}</li>
              <li>{{ t('conceptsGuide.loadBalanceItem2') }}</li>
            </ul>
          </div>
          <div class="bg-white/50 dark:bg-white/5 p-4 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)] shadow-sm">
            <h4 class="font-bold text-[#262624] dark:text-[#f1ead8] flex items-center gap-2">
              {{ t('conceptsGuide.fixedOrder') }}
            </h4>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-1">
              {{ t('conceptsGuide.fixedOrderText') }}
            </p>
          </div>
        </div>
        
        <div class="space-y-4">
          <img
            loading="lazy"
            src="/guide/concepts-priority-1.webp"
            :alt="`${t('conceptsGuide.priorityManagementTitle')} 1`"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
          <img
            loading="lazy"
            src="/guide/concepts-priority-2.webp"
            :alt="`${t('conceptsGuide.priorityManagementTitle')} 2`"
            class="rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)] shadow-sm w-full"
          >
        </div>
      </div>
    </section>
  </div>
</template>
