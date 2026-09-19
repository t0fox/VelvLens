import { contextBridge, ipcRenderer } from 'electron';

contextBridge.exposeInMainWorld('velvLens', {
  retryBackend: () => ipcRenderer.invoke('backend:retry'),
});

window.addEventListener('DOMContentLoaded', () => {
  const retryButton = document.querySelector('#retry');
  if (!retryButton) return;
  retryButton.addEventListener('click', async () => {
    retryButton.setAttribute('disabled', 'true');
    retryButton.textContent = 'Запуск…';
    const result = await window.velvLens.retryBackend();
    if (!result?.ok) {
      retryButton.removeAttribute('disabled');
      retryButton.textContent = 'Повторить запуск';
    }
  });

  const code = new URLSearchParams(window.location.search).get('code');
  const details = {
    STARTUP_TIMEOUT: ['Тайм-аут запуска', 'Локальный resolver не ответил вовремя.'],
    BACKEND_EXITED: ['Resolver завершился', 'Локальный resolver завершился до готовности.'],
    BACKEND_SPAWN_FAILED: ['Ошибка запуска', 'Не удалось запустить локальный resolver.'],
    BACKEND_START_FAILED: ['Ошибка запуска', 'Не удалось запустить локальный resolver.'],
  }[code] ?? ['Ошибка запуска', 'Не удалось запустить локальный resolver.'];
  const category = document.querySelector('#category');
  const detail = document.querySelector('#detail');
  if (category) category.textContent = details[0];
  if (detail) detail.textContent = details[1];
});
