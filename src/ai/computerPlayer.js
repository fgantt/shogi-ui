let aiWorker;

function initializeWorker() {
  aiWorker = new Worker(new URL('./ai.worker.js', import.meta.url), { type: 'module' });

  aiWorker.onmessage = (event) => {
    // The worker will send back the best move, which we will then resolve.
    if (aiWorker.resolve) {
      aiWorker.resolve(event.data);
    }
  };

  aiWorker.onerror = (error) => {
    console.error("AI Worker Error:", error);
    if (aiWorker.reject) {
      aiWorker.reject(error);
    }
  };
}

export function getAiMove(gameState, difficulty) {
  return new Promise((resolve, reject) => {
    if (!aiWorker) {
      initializeWorker();
    }

    aiWorker.resolve = resolve;
    aiWorker.reject = reject;

    aiWorker.postMessage({ gameState, difficulty });
  });
}
