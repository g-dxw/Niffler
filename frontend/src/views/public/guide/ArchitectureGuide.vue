<script setup lang="ts">
import { Network, ArrowRight } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { panelClasses } from './guide-config'
import ArchitectureDiagram from './components/ArchitectureDiagram.vue'

const { t } = useI18n()
</script>

<template>
  <div class="space-y-12">
    <!-- Hero 区域 -->
    <div class="space-y-4">
      <div class="inline-flex items-center gap-1.5 rounded-full bg-[#cc785c]/10 dark:bg-[#cc785c]/20 border border-[#cc785c]/20 dark:border-[#cc785c]/40 px-3 py-1 text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
        <Network class="h-3 w-3" />
        {{ t('architectureGuide.badge') }}
      </div>
      <h1 class="text-3xl font-bold text-[#262624] dark:text-[#f1ead8]">
        {{ t('architectureGuide.title') }}
      </h1>
    </div>

    <!-- 动态架构图 -->
    <div class="mt-10 mb-6">
      <ArchitectureDiagram />
    </div>

    <section>
      <div
        :class="[panelClasses.card]"
        class="p-6 mt-6"
      >
        <div class="space-y-5">
          <!-- 1. 多格式兼容接入 -->
          <div class="space-y-1.5">
            <div class="flex items-center gap-2.5">
              <span class="text-xs font-medium font-mono text-[#cc785c]">1.</span>
              <span class="text-sm font-medium text-[#262624] dark:text-[#f1ead8]">{{ t('architectureGuide.multiFormat') }}</span>
            </div>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] leading-relaxed pl-[22px]">
              {{ t('architectureGuide.multiFormatText') }}
            </p>
          </div>

          <!-- 2. 统一模型路由 -->
          <div class="space-y-1.5">
            <div class="flex items-center gap-2.5">
              <span class="text-xs font-medium font-mono text-[#cc785c]">2.</span>
              <span class="text-sm font-medium text-[#262624] dark:text-[#f1ead8]">{{ t('architectureGuide.unifiedRouting') }}</span>
            </div>
            <p class="text-sm text-[#666663] dark:text-[#a3a094] leading-relaxed pl-[22px]">
              {{ t('architectureGuide.unifiedRoutingText') }}
            </p>
          </div>

          <!-- 3. 跨平台格式转换 -->
          <div class="space-y-2">
            <div class="flex items-center gap-2.5">
              <span class="text-xs font-medium font-mono text-[#cc785c]">3.</span>
              <span class="text-sm font-medium text-[#262624] dark:text-[#f1ead8]">{{ t('architectureGuide.crossFormat') }}</span>
            </div>
            <div class="flex flex-col md:flex-row items-stretch md:items-center gap-2 bg-[#f5f5f0]/50 dark:bg-[rgba(227,224,211,0.05)] p-3 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)]">
              <div class="flex-1 bg-white dark:bg-[#191714] border border-[#e5e4df] dark:border-white/10 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#262624] dark:text-white/80">
                {{ t('architectureGuide.compatibleInput') }}
              </div>
              <ArrowRight class="hidden md:block w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 shrink-0" />
              <div class="md:hidden flex justify-center">
                <ArrowRight class="w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 rotate-90" />
              </div>
              <div class="flex-1 bg-[#cc785c]/5 dark:bg-[#cc785c]/10 border border-[#cc785c]/15 dark:border-[#cc785c]/20 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
                {{ t('architectureGuide.formatConversion') }}
              </div>
              <ArrowRight class="hidden md:block w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 shrink-0" />
              <div class="md:hidden flex justify-center">
                <ArrowRight class="w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 rotate-90" />
              </div>
              <div class="flex-1 bg-white dark:bg-[#191714] border border-[#e5e4df] dark:border-white/10 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#262624] dark:text-white/80">
                {{ t('architectureGuide.upstreamProvider') }}
              </div>
              <ArrowRight class="hidden md:block w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 shrink-0" />
              <div class="md:hidden flex justify-center">
                <ArrowRight class="w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 rotate-90" />
              </div>
              <div class="flex-1 bg-[#cc785c]/5 dark:bg-[#cc785c]/10 border border-[#cc785c]/15 dark:border-[#cc785c]/20 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
                {{ t('architectureGuide.formatConversion') }}
              </div>
              <ArrowRight class="hidden md:block w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 shrink-0" />
              <div class="md:hidden flex justify-center">
                <ArrowRight class="w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 rotate-90" />
              </div>
              <div class="flex-1 bg-white dark:bg-[#191714] border border-[#e5e4df] dark:border-white/10 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#262624] dark:text-white/80">
                {{ t('architectureGuide.compatibleResponse') }}
              </div>
            </div>
          </div>

          <!-- 4. 同生态原生透传 -->
          <div class="space-y-2">
            <div class="flex items-center gap-2.5">
              <span class="text-xs font-medium font-mono text-[#cc785c]">4.</span>
              <span class="text-sm font-medium text-[#262624] dark:text-[#f1ead8]">{{ t('architectureGuide.nativePassthrough') }}</span>
            </div>
            <div class="flex flex-col md:flex-row items-stretch md:items-center gap-2 bg-[#f5f5f0]/50 dark:bg-[rgba(227,224,211,0.05)] p-3 rounded-xl border border-[#e5e4df] dark:border-[rgba(227,224,211,0.06)]">
              <div class="flex-1 bg-white dark:bg-[#191714] border border-[#e5e4df] dark:border-white/10 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#262624] dark:text-white/80">
                {{ t('architectureGuide.nativeInput') }}
              </div>
              <ArrowRight class="hidden md:block w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 shrink-0" />
              <div class="md:hidden flex justify-center">
                <ArrowRight class="w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 rotate-90" />
              </div>
              <div class="flex-[2] bg-[#cc785c]/5 dark:bg-[#cc785c]/10 border border-[#cc785c]/15 dark:border-[#cc785c]/20 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#cc785c] dark:text-[#d4a27f]">
                {{ t('architectureGuide.nativeFlow') }}
              </div>
              <ArrowRight class="hidden md:block w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 shrink-0" />
              <div class="md:hidden flex justify-center">
                <ArrowRight class="w-4 h-4 text-[#cc785c]/40 dark:text-[#cc785c]/30 rotate-90" />
              </div>
              <div class="flex-1 bg-white dark:bg-[#191714] border border-[#e5e4df] dark:border-white/10 rounded-lg py-2 px-3 text-center text-xs font-medium text-[#262624] dark:text-white/80">
                {{ t('architectureGuide.nativeOutput') }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
