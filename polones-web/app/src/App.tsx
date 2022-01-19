import React, { DragEvent, MouseEvent } from 'react';

import './App.css';

declare global {
  interface Window {
    polones_display_draw(frame: Uint8ClampedArray): void;
  }
}

async function importPolones() {
  return import('polones-web');
}

type PromiseResult<T> = T extends (() => Promise<infer U>) ? U : never;
type PolonesModule = PromiseResult<typeof importPolones>;

function App() {
  const [polones, setPolones] = React.useState<PolonesModule | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [viewportSize, setViewportSize] = React.useState<[number, number]>([
    window.visualViewport.width,
    window.visualViewport.height
  ]);
  const [state, setState] = React.useState<'rom' | 'running' | 'paused'>('rom');
  const [gameInterval, setGameInterval] = React.useState<number | null>();
  const canvasRef = React.useRef<HTMLCanvasElement | null>(null);

  React.useEffect(() => {
    importPolones()
      .then(polones => {
        setPolones(polones);

        window.onresize = event => {
          setViewportSize([
            window.visualViewport.width,
            window.visualViewport.height,
          ]);
        };

        window.polones_display_draw = function (frame: Uint8ClampedArray) {
          canvasRef
            .current
            ?.getContext('2d')
            ?.putImageData(new ImageData(frame, 256, 240), 0, 0);
        };
      })
      .catch(error => setError(error));
  }, []);

  function handleDrop(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();

    let file: File | undefined = undefined;

    if (event.dataTransfer.items) {
      for (const item of event.dataTransfer.items) {
        if (item.kind === 'file') {
          let f = item.getAsFile();
          if (f) {
            file = f;
            break;
          }
        }
      }
    } else {
      for (const f of event.dataTransfer.files) {
        file = f;
        break;
      }
    }

    if (file) {
      file.arrayBuffer()
        .then(rom => {
          let error = polones!.polones_start(new Uint8Array(rom));
          if (error) {
            setError(error);
            setState('rom');
            setGameInterval(null);
          } else {
            setError(null);
            setState('running');
            setGameInterval(startInterval());
          }
        })
        .catch(error => setError(error));
    } else {
      setError("No ROM?");
    }
  }

  function startInterval(): number {
    const ref = { current: 0 };
    ref.current = window.setInterval(function runTicksForOneFrame() {
      try {
        for (let i = 0; i < 29829; i++) {
          polones!.polones_tick();
        }
      } catch (e) {
        window.clearInterval(ref.current);
        console.error(e);
      }
    }, 1000/60);
    return ref.current;
  }

  function handleDragOver(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
  }

  function handlePauseClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state === 'running') {
      window.clearInterval(gameInterval!);
      setGameInterval(null);
      setState('paused');
    }
  }

  function handleUnpauseClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state === 'paused') {
      setGameInterval(startInterval());
      setState('running');
    }
  }

  function handleStopClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state !== 'rom') {
      window.clearInterval(gameInterval!);
      setGameInterval(null);
      setState('rom');
    }
  }

  let viewportRatio = viewportSize[0] / viewportSize[1];
  let canvasRatio = 256 / 240;
  let zoom: number;
  if (viewportRatio > canvasRatio) {
    // --------------
    // |    ----    |
    // |    |  |    |
    // |    ----    |
    // --------------
    zoom = viewportSize[1] / 240;
  } else {
    // --------
    // |      |
    // | ---- |
    // | |  | |
    // | ---- |
    // |      |
    // --------
    zoom = viewportSize[0] / 256;
  }
  let transform = `scale(${zoom})`;

  return (
    <div className="App">
      <div className="center">
        {polones && state === 'rom' && (
          <div id="rom" className="rom-input" onDrop={handleDrop} onDragOver={handleDragOver}>
            Drop a ROM here!
          </div>
        )}
        {polones && (state !== 'rom') && (
          <canvas ref={canvasRef} width={256} height={240} className="canvas" style={{ transform }}></canvas>
        )}
        {error && (
          <div className="error">{error}</div>
        )}

        <aside className="toolbar">
          {state === 'running' && (
            <button type="button" onClick={handlePauseClick}>⏸︎</button>
          )}
          {state === 'paused' && (
            <button type="button" onClick={handleUnpauseClick}>⏵︎</button>
          )}
          {state !== 'rom' && (
            <button type="button" onClick={handleStopClick}>&times;</button>
          )}
        </aside>
      </div>
    </div>
  );
}

export default App;
