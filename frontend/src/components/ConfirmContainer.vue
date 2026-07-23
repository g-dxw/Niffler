<template>
  <AlertDialog
    :model-value="state.isOpen"
    :title="state.title || t('confirmContainer.title')"
    :description="state.message"
    :confirm-text="state.confirmText || t('confirmContainer.confirm')"
    :cancel-text="state.cancelText || t('common.cancel')"
    :type="state.variant || 'question'"
    @update:model-value="handleClose"
    @confirm="handleConfirm"
    @cancel="handleCancel"
  />
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import AlertDialog from './common/AlertDialog.vue'
import { useConfirm } from '@/composables/useConfirm'

const { state, handleConfirm, handleCancel } = useConfirm()
const { t } = useI18n()

function handleClose(value: boolean) {
  if (!value) {
    handleCancel()
  }
}
</script>
