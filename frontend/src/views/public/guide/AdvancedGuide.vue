<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <div class="space-y-12 pb-12">
    <!-- Hero 区域 -->
    <div class="space-y-4">
      <div class="inline-flex items-center gap-1.5 rounded-full bg-[#cc785c]/10 dark:bg-[#cc785c]/20 border border-[#cc785c]/20 dark:border-[#cc785c]/40 px-3 py-1 text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
        <Settings class="h-3 w-3" />
        {{ t('advancedGuide.badge') }}
      </div>
      <h1 class="text-3xl font-bold text-[#262624] dark:text-[#f1ead8]">
        {{ t('advancedGuide.title') }}
      </h1>
      <p class="text-base text-[#666663] dark:text-[#a3a094] max-w-2xl">
        {{ t('advancedGuide.description') }}
      </p>
    </div>

    <!-- 1. 格式转换 -->
    <section
      id="format-conversion"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('advancedGuide.formatTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('advancedGuide.formatText') }}
      </p>
      <ul class="list-decimal pl-5 space-y-2 mt-2 text-[#666663] dark:text-[#a3a094] text-sm">
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('advancedGuide.globalConversion') }}：</strong> {{ t('advancedGuide.globalConversionText') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('advancedGuide.providerConversion') }}：</strong> {{ t('advancedGuide.providerConversionText') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('advancedGuide.endpointConversion') }}：</strong> {{ t('advancedGuide.endpointConversionText') }}</li>
      </ul>
    </section>

    <!-- 2. 请求上游固定非流/流式 -->
    <section
      id="stream-policy"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('advancedGuide.streamPolicyTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('advancedGuide.streamPolicyText') }}
      </p>
    </section>

    <!-- 3. 请求头/体编辑 -->
    <section
      id="header-body-edit"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('advancedGuide.headerBodyTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('advancedGuide.headerBodyText') }}
      </p>
    </section>

    <!-- 4. 模型映射 -->
    <section
      id="model-mapping"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('advancedGuide.modelMappingTitle') }}</h2>
      <p class="text-sm text-[#666663] dark:text-[#a3a094] mb-4">
        {{ t('advancedGuide.modelMappingText') }}
      </p>
    </section>

    <!-- 5. 正则映射 -->
    <section
      id="regex-mapping"
      class="scroll-mt-24 lg:scroll-mt-20"
    >
      <h2>{{ t('advancedGuide.regexTitle') }}</h2>
      <ul class="list-decimal pl-5 space-y-2 mt-4 text-[#666663] dark:text-[#a3a094] text-sm">
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('advancedGuide.modelPermissions') }}：</strong> {{ t('advancedGuide.modelPermissionsText') }}</li>
        <li><strong class="text-[#262624] dark:text-[#f1ead8] font-medium">{{ t('advancedGuide.autoFetch') }}：</strong> {{ t('advancedGuide.autoFetchText') }}</li>
      </ul>
    </section>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-8 mt-12 pt-8 border-t border-[#e5e4df] dark:border-[rgba(227,224,211,0.12)]">
      <!-- 6. 余额监控 -->
      <section
        id="balance-monitor"
        class="scroll-mt-24 lg:scroll-mt-20"
      >
        <h3 class="mt-0 text-xl text-[#262624] dark:text-[#f1ead8]">
          {{ t('advancedGuide.balanceTitle') }}
        </h3>
        <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-2">
          {{ t('advancedGuide.balanceText') }}
        </p>
      </section>

      <!-- 7. 配置导入/出 -->
      <section
        id="config-export"
        class="scroll-mt-24 lg:scroll-mt-20"
      >
        <h3 class="mt-0 text-xl text-[#262624] dark:text-[#f1ead8]">
          {{ t('advancedGuide.configTitle') }}
        </h3>
        <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-2">
          {{ t('advancedGuide.configText') }}
        </p>
      </section>

      <!-- 8. 锁定用户密钥 -->
      <section
        id="lock-key"
        class="scroll-mt-24 lg:scroll-mt-20"
      >
        <h3 class="mt-0 text-xl text-[#262624] dark:text-[#f1ead8]">
          {{ t('advancedGuide.lockKeyTitle') }}
        </h3>
        <p class="text-sm text-[#666663] dark:text-[#a3a094] mt-2">
          {{ t('advancedGuide.lockKeyText') }}
        </p>
      </section>
    </div>
  </div>
</template>
