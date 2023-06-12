import React from 'react';

// Calculates device's refresh rate by measuring time between draws.
export default function useRefreshRateRef() {
  const lastTimestampRef = React.useRef(0);
  const deltasRef = React.useRef<number[]>([]);
  const refreshRateRef = React.useRef(60);

  const frameCallbackRef = React.useRef<number | null>(null);

  React.useEffect(() => {
    function callback(timestamp: number) {
      const delta = timestamp - lastTimestampRef.current;

      // let's ignore reads over 100 milliseconds
      if (delta < 100) {
        deltasRef.current.push(delta);
        if (deltasRef.current.length > 60) {
          deltasRef.current.shift();
        }
        const refreshRate = Math.round(1000 / (deltasRef.current.reduce((acc, next) => acc + next, 0) / deltasRef.current.length));

        refreshRateRef.current = refreshRate;
      }
      lastTimestampRef.current = timestamp;
      frameCallbackRef.current = window.requestAnimationFrame(callback);
    };
    frameCallbackRef.current = window.requestAnimationFrame(callback);

    return () => {
      window.cancelAnimationFrame(frameCallbackRef.current!);
    };
  });

  return refreshRateRef;
}
