<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps<{
  mode: 'controller' | 'controlled'
}>()

const emit = defineEmits<{
  connected: []
}>()

const connectionId = ref('')
const connectionInfo = ref<any>(null)
const status = ref('')

async function createConnection() {
  try {
    status.value = 'Creating connection...'
    connectionInfo.value = await invoke('create_connection')
    status.value = 'Waiting for connection...'
  } catch (e) {
    status.value = `Error: ${e}`
  }
}

async function acceptConnection() {
  try {
    status.value = 'Accepting connection...'
    const info = await invoke('accept_connection', { info: JSON.parse(connectionId.value) })
    await invoke('complete_connection', { answerInfo: info })
    emit('connected')
  } catch (e) {
    status.value = `Error: ${e}`
  }
}
</script>

<template>
  <div class="connection-view">
    <!-- 控制端界面 -->
    <div v-if="mode === 'controller'" class="controller-view">
      <fluent-button appearance="accent" @click="createConnection">
        Create Connection
      </fluent-button>
      
      <div v-if="connectionInfo" class="connection-info">
        <h3>Connection Information</h3>
        <p>Share this with the controlled device:</p>
        <fluent-text-field
          readonly
          :value="JSON.stringify(connectionInfo)"
        ></fluent-text-field>
        <fluent-button @click="navigator.clipboard.writeText(JSON.stringify(connectionInfo))">
          Copy
        </fluent-button>
      </div>
    </div>

    <!-- 被控端界面 -->
    <div v-else class="controlled-view">
      <h3>Enter Connection Information</h3>
      <fluent-text-field
        v-model="connectionId"
        placeholder="Paste connection info here..."
      ></fluent-text-field>
      <fluent-button appearance="accent" @click="acceptConnection">
        Accept Connection
      </fluent-button>
    </div>

    <p class="status">{{ status }}</p>
  </div>
</template>

<style scoped>
.connection-view {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}

.connection-info {
  margin-top: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.status {
  margin-top: 1rem;
  font-style: italic;
}

fluent-button {
  margin-top: 1rem;
}

fluent-text-field {
  width: 100%;
}
</style> 