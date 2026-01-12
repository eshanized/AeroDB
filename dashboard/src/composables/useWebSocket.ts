import { ref } from 'vue'

const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:54321/realtime/v1'

export const useWebSocket = (channel?: string) => {
    const connected = ref(false)
    const events = ref<Array<{ type: string; payload: unknown; timestamp: string }>>([])
    const error = ref<string | null>(null)
    let ws: WebSocket | null = null

    const connect = () => {
        try {
            ws = new WebSocket(wsUrl)

            ws.onopen = () => {
                connected.value = true
                error.value = null

                if (channel) {
                    subscribe(channel)
                }
            }

            ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data)
                    events.value.unshift({
                        ...data,
                        timestamp: new Date().toISOString(),
                    })

                    // Keep only last 100 events
                    if (events.value.length > 100) {
                        events.value = events.value.slice(0, 100)
                    }
                } catch (err) {
                    console.error('Failed to parse WebSocket message:', err)
                }
            }

            ws.onerror = (err) => {
                error.value = 'WebSocket error occurred'
                console.error('WebSocket error:', err)
            }

            ws.onclose = () => {
                connected.value = false
            }
        } catch (err) {
            error.value = err instanceof Error ? err.message : 'Failed to connect'
        }
    }

    const disconnect = () => {
        if (ws) {
            ws.close()
            ws = null
            connected.value = false
        }
    }

    const subscribe = (channelName: string, filter?: Record<string, unknown>) => {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({
                type: 'subscribe',
                channel: channelName,
                filter,
            }))
        }
    }

    const unsubscribe = (channelName: string) => {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({
                type: 'unsubscribe',
                channel: channelName,
            }))
        }
    }

    const send = (message: unknown) => {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(message))
        }
    }

    return {
        connected,
        events,
        error,
        connect,
        disconnect,
        subscribe,
        unsubscribe,
        send,
    }
}
