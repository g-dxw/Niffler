<template>
  <section class="rounded-lg border border-border bg-muted/30 p-3">
    <p class="mb-2 text-xs font-medium text-foreground">
      演示账号
    </p>
    <div class="grid gap-2 sm:grid-cols-2">
      <button
        type="button"
        class="demo-account flex min-h-9 items-center gap-2 px-3"
        :disabled="disabled"
        :data-state="disabled ? 'disabled' : 'idle'"
        @click="$emit('fill', 'admin')"
      >
        <span class="demo-account__mark flex h-4 w-4 items-center justify-center text-[10px] font-bold">A</span>
        <span>管理员</span>
      </button>
      <button
        type="button"
        class="demo-account flex min-h-9 items-center gap-2 px-3"
        :disabled="disabled"
        :data-state="disabled ? 'disabled' : 'idle'"
        @click="$emit('fill', 'user')"
      >
        <span class="demo-account__mark flex h-4 w-4 items-center justify-center text-[10px] font-bold">U</span>
        <span>普通用户</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
defineProps<{
  disabled?: boolean
}>()

defineEmits<{
  fill: [type: 'admin' | 'user']
}>()
</script>

<style scoped>
.demo-account {
  border-radius: var(--radius);
  color: var(--muted-foreground);
  background: var(--background);
  border: 1px solid var(--border);
  transition:
    color 160ms ease,
    border-color 160ms ease,
    background-color 160ms ease,
    transform 160ms ease;
}

.demo-account:hover {
  color: var(--foreground);
  border-color: var(--primary);
}

.demo-account:active {
  transform: translateY(1px);
}

.demo-account:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.demo-account__mark {
  border-radius: calc(var(--radius) - 0.125rem);
  background: var(--muted);
  color: var(--foreground);
}
</style>
