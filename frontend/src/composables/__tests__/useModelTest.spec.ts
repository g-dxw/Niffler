import { describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h } from '@/test/vue'
import { useModelTest } from '@/composables/useModelTest'

const apiMocks = vi.hoisted(() => ({
  testModel: vi.fn(),
  testModelFailover: vi.fn(),
  getRequestTrace: vi.fn(),
}))

vi.mock('@/api/endpoints/providers', () => ({
  testModel: apiMocks.testModel,
  testModelFailover: apiMocks.testModelFailover,
}))

vi.mock('@/api/requestTrace', () => ({
  requestTraceApi: {
    getRequestTrace: apiMocks.getRequestTrace,
  },
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}))

describe('useModelTest', () => {
  it('sends api_key_id for a direct single-account test', async () => {
    apiMocks.testModel.mockResolvedValue({
      success: true,
      model: 'gpt-5.1',
      provider: { id: 'provider-1', name: 'Provider 1' },
      data: { response: { status_code: 200 } },
    })
    apiMocks.getRequestTrace.mockResolvedValue({
      request_id: 'trace-1',
      candidates: [],
    })

    const exposed: { startTest?: ReturnType<typeof useModelTest>['startTest'] } = {}
    const Harness = defineComponent({
      setup() {
        const modelTest = useModelTest({ providerId: () => 'provider-1' })
        exposed.startTest = modelTest.startTest
        return () => h('div')
      },
    })
    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp(Harness)
    app.mount(root)

    await exposed.startTest?.({
      mode: 'direct',
      modelName: 'gpt-5.1',
      displayLabel: 'OpenAI / gpt-5.1',
      apiFormat: 'openai:responses',
      endpointId: 'endpoint-1',
      endpointBaseUrl: 'https://example.com',
      apiKeyId: 'provider-key-1',
      requestBody: {
        model: 'gpt-5.1',
        input: 'hello',
      },
    })

    expect(apiMocks.testModel).toHaveBeenCalledWith(
      expect.objectContaining({
        provider_id: 'provider-1',
        model_name: 'gpt-5.1',
        mode: 'direct',
        api_key_id: 'provider-key-1',
        endpoint_id: 'endpoint-1',
      }),
      expect.any(Object),
    )
    expect(apiMocks.testModelFailover).not.toHaveBeenCalled()

    app.unmount()
    root.remove()
  })
})
