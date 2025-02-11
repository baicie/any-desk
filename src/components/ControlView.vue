<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps<{
  mode: 'controller' | 'controlled'
}>()

const emit = defineEmits<{
  disconnect: []
}>()

const videoRef = ref<HTMLVideoElement>()

// 处理鼠标事件
function handleMouseMove(e: MouseEvent) {
  if (props.mode === 'controller') {
    invoke('send_input', {
      event: {
        type: 'MouseMove',
        x: e.clientX,
        y: e.clientY
      }
    })
  }
}

function handleMouseClick(e: MouseEvent) {
  if (props.mode === 'controller') {
    invoke('send_input', {
      event: {
        type: 'MouseClick',
        button: e.button === 0 ? 'Left' : 'Right'
      }
    })
  }
}

function disconnect() {
  emit('disconnect')
}

onMounted(() => {
  if (videoRef.value) {
    // 设置视频流
  }
})

onUnmounted(() => {
  // 清理资源
})
</script>

<template>
  <div class="control-view">
    <div class="toolbar">
      <fluent-button appearance="accent" @click="disconnect">
        Disconnect
      </fluent-button>
    </div>

    <div class="screen-view"
      @mousemove="handleMouseMove"
      @click="handleMouseClick">
      <video ref="videoRef" autoplay></video>
    </div>
  </div>
</template>

<style scoped>
.control-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.toolbar {
  padding: 0.5rem;
  border-bottom: 1px solid #ccc;
}

.screen-view {
  flex: 1;
  position: relative;
  background: #000;
}

video {
  width: 100%;
  height: 100%;
  object-fit: contain;
}
</style> 