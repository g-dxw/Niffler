<template>
  <div
    class="app-shell"
    :class="{ 'pt-24': showNotice }"
  >
    <div
      v-if="showNotice"
      class="fixed top-6 left-0 right-0 z-50 flex justify-center px-4"
    >
      <slot name="notice" />
    </div>

    <div class="app-shell__backdrop">
      <div class="app-shell__gradient app-shell__gradient--primary" />
      <div class="app-shell__gradient app-shell__gradient--accent" />
    </div>

    <div class="app-shell__body">
      <aside
        v-if="$slots.sidebar"
        class="app-shell__sidebar"
        :class="sidebarClass"
      >
        <slot name="sidebar" />
      </aside>

      <button
        v-if="mobileSidebarOpen"
        type="button"
        class="fixed inset-0 z-50 bg-black/35 backdrop-blur-[1px] lg:hidden"
      :aria-label="t('appShell.closeSidebar')"
        @click="emit('close-mobile-sidebar')"
      />

      <div
        class="app-shell__content"
        :class="contentClass"
      >
        <slot name="header" />
        <main :class="mainClass">
          <slot />
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  showNotice?: boolean
  contentClass?: string
  mainClass?: string
  sidebarClass?: string
  mobileSidebarOpen?: boolean
}>(), {
  showNotice: false,
  contentClass: '',
  mainClass: '',
  sidebarClass: '',
  mobileSidebarOpen: false,
})

const emit = defineEmits<{
  (event: 'close-mobile-sidebar'): void
}>()

const showNotice = computed(() => props.showNotice)

// contentClass and mainClass are now just the props, base classes are in template
const contentClass = computed(() => props.contentClass)
const mainClass = computed(() => ['app-shell__main', props.mainClass].filter(Boolean).join(' '))
const sidebarClass = computed(() => props.sidebarClass)
const mobileSidebarOpen = computed(() => props.mobileSidebarOpen)
</script>
