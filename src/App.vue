<template>
  <div class="app">
    <div class="titlebar">
      <span class="app-icon">⚡</span>
      <span class="app-name">Process Manager</span>
    </div>
    <div class="workspace">
      <div class="sidebar-wrap">
        <ProcessList />
      </div>
      <div class="main-wrap">
        <ProcessPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useProcessStore } from "@/stores/processStore";
import ProcessList from "@/components/ProcessList.vue";
import ProcessPanel from "@/components/ProcessPanel.vue";

const store = useProcessStore();

onMounted(async () => {
  try {
    await store.loadConfig();
  } catch {
    // Fresh install - no config yet
  }

  setInterval(async () => {
    try { await store.loadProcesses(); } catch { /* ignore */ }
  }, 2000);
});
</script>

<style>
* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background: #171717;
  color: #e2e8f0;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 100%;
}

.app {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  overflow: hidden;
}

.titlebar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px;
  height: 36px;
  background: #111;
  border-bottom: 1px solid #2e2e2e;
  flex-shrink: 0;
  user-select: none;
}
.app-icon { font-size: 0.9rem; }
.app-name {
  font-size: 0.82rem;
  font-weight: 600;
  color: #6b7280;
  letter-spacing: 0.03em;
}

.workspace {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar-wrap {
  width: 280px;
  min-width: 220px;
  flex-shrink: 0;
  border-right: 1px solid #2e2e2e;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.main-wrap {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  height: 100%;
}

::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #374151; border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: #4b5563; }
</style>