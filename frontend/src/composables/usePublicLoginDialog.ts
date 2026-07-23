import { ref } from 'vue'

const showPublicLoginDialog = ref(false)

export function usePublicLoginDialog() {
  return { showLoginDialog: showPublicLoginDialog }
}
