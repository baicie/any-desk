<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import ConnectionView from './components/ConnectionView.vue'
import ControlView from './components/ControlView.vue'

const mode = ref<'controller' | 'controlled'>('controller')
const isConnected = ref(false)

onMounted(async () => {
  mode.value = await invoke('get_app_mode')
})
</script>

<template>
  <div class="container">
    <fluent-card>
      <h1>{{ mode === 'controller' ? 'AnyDesk - Controller' : 'AnyDesk - Controlled' }}</h1>
      <fluent-divider></fluent-divider>
      
      <ConnectionView v-if="!isConnected" 
        :mode="mode"
        @connected="isConnected = true" />
      
      <ControlView v-else 
        :mode="mode"
        @disconnect="isConnected = false" />
    </fluent-card>
  </div>
</template>

<style>
.container {
  margin: 0;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

fluent-card {
  padding: 2rem;
  border-radius: 8px;
}

h1 {
  font-size: 2rem;
  margin-bottom: 1rem;
}
</style>