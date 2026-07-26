<template>
  <!-- 用户数据导入对话框 -->
  <Dialog
    :open="importUsersDialogOpen"
    :title="t('importDialogs.usersTitle')"
    :description="t('importDialogs.hint')"
    @update:open="$emit('update:importUsersDialogOpen', $event)"
  >
    <div class="space-y-4">
      <div
        v-if="importUsersPreview"
        class="text-sm"
      >
        <p class="font-medium mb-2">
          {{ t('importDialogs.preview') }}
        </p>
        <ul class="space-y-1 text-muted-foreground">
          <li v-if="importUsersPreview.user_groups?.length">
            {{ t('importDialogs.groups') }}: {{ importUsersPreview.user_groups.length }}
          </li>
          <li>{{ t('importDialogs.users') }}: {{ importUsersPreview.users?.length || 0 }}</li>
          <li>
            {{ t('importDialogs.keys') }}: {{ importUsersPreview.users?.reduce((sum: number, u: { api_keys?: unknown[] }) => sum + (u.api_keys?.length || 0), 0) }}
          </li>
          <li v-if="importUsersPreview.standalone_keys?.length">
            {{ t('importDialogs.standalone') }}: {{ importUsersPreview.standalone_keys.length }}
          </li>
        </ul>
      </div>

      <div>
        <Label class="block text-sm font-medium mb-2">{{ t('importDialogs.conflict') }}</Label>
        <Select
          :model-value="usersMergeMode"
          :open="usersMergeModeSelectOpen"
          @update:model-value="$emit('update:usersMergeMode', $event as 'skip' | 'overwrite' | 'error')"
          @update:open="$emit('update:usersMergeModeSelectOpen', $event)"
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="skip">
              {{ t('importDialogs.skip') }}
            </SelectItem>
            <SelectItem value="overwrite">
              {{ t('importDialogs.overwrite') }}
            </SelectItem>
            <SelectItem value="error">
              {{ t('importDialogs.abort') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="mt-1 text-xs text-muted-foreground">
          <template v-if="usersMergeMode === 'skip'">
            {{ t('importDialogs.skipHint') }}
          </template>
          <template v-else-if="usersMergeMode === 'overwrite'">
            {{ t('importDialogs.overwriteHint') }}
          </template>
          <template v-else>
            {{ t('importDialogs.abortHint') }}
          </template>
        </p>
      </div>

      <p class="text-xs text-muted-foreground">
        {{ t('importDialogs.warning') }}
      </p>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="$emit('update:importUsersDialogOpen', false); $emit('update:usersMergeModeSelectOpen', false)"
      >
        {{ t('importDialogs.cancel') }}
      </Button>
      <Button
        :disabled="importUsersLoading"
        @click="$emit('confirm')"
      >
        {{ importUsersLoading ? t('importDialogs.importing') : t('importDialogs.confirm') }}
      </Button>
    </template>
  </Dialog>

  <!-- 用户数据导入结果对话框 -->
  <Dialog
    :open="importUsersResultDialogOpen"
    :title="t('importDialogs.complete')"
    @update:open="$emit('update:importUsersResultDialogOpen', $event)"
  >
    <div
      v-if="importUsersResult"
      class="space-y-4"
    >
      <div class="grid grid-cols-2 gap-4 text-sm">
        <div v-if="importUsersResult.stats.user_groups">
          <p class="font-medium">
            {{ t('importDialogs.groups') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importUsersResult.stats.user_groups.created }},
            {{ t('importDialogs.updated') }}: {{ importUsersResult.stats.user_groups.updated }},
            {{ t('importDialogs.skipped') }}: {{ importUsersResult.stats.user_groups.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('importDialogs.users') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importUsersResult.stats.users.created }},
            {{ t('importDialogs.updated') }}: {{ importUsersResult.stats.users.updated }},
            {{ t('importDialogs.skipped') }}: {{ importUsersResult.stats.users.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            API Keys
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importUsersResult.stats.api_keys.created }},
            {{ t('importDialogs.skipped') }}: {{ importUsersResult.stats.api_keys.skipped }}
          </p>
        </div>
        <div
          v-if="importUsersResult.stats.standalone_keys"
          class="col-span-2"
        >
          <p class="font-medium">
            {{ t('importDialogs.standalone') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importUsersResult.stats.standalone_keys.created }},
            {{ t('importDialogs.skipped') }}: {{ importUsersResult.stats.standalone_keys.skipped }}
          </p>
        </div>
      </div>

      <div
        v-if="importUsersResult.stats.errors.length > 0"
        class="p-3 bg-destructive/10 rounded-lg"
      >
        <p class="font-medium text-destructive mb-2">
          {{ t('importDialogs.warnings') }}
        </p>
        <ul class="text-sm text-destructive space-y-1">
          <li
            v-for="(err, index) in importUsersResult.stats.errors"
            :key="index"
          >
            {{ err }}
          </li>
        </ul>
      </div>
    </div>

    <template #footer>
      <Button @click="$emit('update:importUsersResultDialogOpen', false)">
        {{ t('importDialogs.confirmDone') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import Button from '@/components/ui/button.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { Dialog } from '@/components/ui'
import type { UsersExportData, UsersImportResponse } from '@/api/admin'

defineProps<{
  importUsersDialogOpen: boolean
  importUsersResultDialogOpen: boolean
  importUsersPreview: UsersExportData | null
  importUsersResult: UsersImportResponse | null
  usersMergeMode: 'skip' | 'overwrite' | 'error'
  usersMergeModeSelectOpen: boolean
  importUsersLoading: boolean
}>()

defineEmits<{
  confirm: []
  'update:importUsersDialogOpen': [value: boolean]
  'update:importUsersResultDialogOpen': [value: boolean]
  'update:usersMergeMode': [value: 'skip' | 'overwrite' | 'error']
  'update:usersMergeModeSelectOpen': [value: boolean]
}>()
</script>
