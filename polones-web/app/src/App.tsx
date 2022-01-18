import React, { DragEvent } from 'react';

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
  const [running, setRunning] = React.useState(false);
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
            setRunning(false);
            setError(error);
          } else {
            setRunning(true);
            setError(null);
            window.setInterval(() => {
              for (let i = 0; i < 29829; i++) {
                polones!.polones_tick();
              }
            }, 1000/60);
          }
        })
        .catch(error => setError(error));
    } else {
      setError("No ROM?");
    }
  }

  function handleDragOver(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
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
        {polones && !running && (
          <div id="rom" className="rom-input" onDrop={handleDrop} onDragOver={handleDragOver}>
            Drop a ROM here!
          </div>
        )}
        {polones && running && (
          <canvas ref={canvasRef} width={256} height={240} className="canvas" style={{ transform }}></canvas>
        )}
        {error && (
          <div className="error">{error}</div>
        )}
      </div>
    </div>
  );
}

export default App;
