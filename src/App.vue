<script setup lang="ts">
import BottomView from "./components/BottomView.vue";
import ListView from "./components/ListView.vue";
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { DragDropEvent } from "@tauri-apps/api/window";
import type { Event } from "@tauri-apps/api/event";
import Preference from "./components/Preference.vue";

const paths = ref<string[]>([]);
const showPreference = ref(false);

let unlistenDragDrop: (() => void) | undefined;

onMounted(async () => {
  const win = getCurrentWindow();
  unlistenDragDrop = await win.onDragDropEvent((e: Event<DragDropEvent>) => {
    const p = e.payload;
    if (p.type === "drop") {
      for (const filePath of p.paths) {
        if (!paths.value.includes(filePath)) {
          paths.value.push(filePath);
        }
      }
    }
  });
});

onUnmounted(() => {
  unlistenDragDrop?.();
});

const openPreference = () => {
  showPreference.value = true;
};

const closePreference = () => {
  showPreference.value = false;
};
</script>

<template>
  <div class="container">
    <list-view :paths="paths.slice().reverse()" />
    <bottom-view @open-preference="openPreference" />
    <preference :show="showPreference" @done="closePreference" />
  </div>
</template>

<style scoped lang="less">
.container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-image: linear-gradient(to bottom, #4ea6d2, #3777b6);
  overflow: hidden;
}
</style>
