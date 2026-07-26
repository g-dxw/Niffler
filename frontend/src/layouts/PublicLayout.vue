<template>
  <div class="flex min-h-screen flex-col bg-background text-foreground literary-grid literary-paper">
    <PublicHeader @login="showLoginDialog = true" />
    <div class="flex-1">
      <RouterView />
    </div>
    <PublicFooter />
    <LoginDialog v-model="showLoginDialog" />
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import PublicHeader from '@/components/common/PublicHeader.vue'
import PublicFooter from '@/components/common/PublicFooter.vue'
import LoginDialog from '@/features/auth/components/LoginDialog.vue'
import { usePublicLoginDialog } from '@/composables/usePublicLoginDialog'

const { showLoginDialog } = usePublicLoginDialog()
</script>
