import React, { DragEvent, MouseEvent } from 'react';
import InputScreen from './InputScreen';

import './App.css';
import { InputMapping } from './types';

declare global {
  interface Window {
    keyboardState: { [key: string]: boolean },
    isPressed(path: string): boolean,
    polones_display_draw(frame: Uint8ClampedArray): void,
    polones_input_read_port_1(): string,
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
  const [inputMapping, setInputMapping] = React.useState<InputMapping>((() => {
    let inputMapping = window.localStorage.getItem('inputMapping');
    if (inputMapping) {
      return JSON.parse(inputMapping);
    } else {
      return {
        port1: {
          type: 'unplugged',
        },
        port2: {
          type: 'unplugged',
        }
      };
    }
  })());
  const inputMappingRef = React.useRef<InputMapping>(inputMapping);
  const [gameInterval, setGameInterval] = React.useState<number | null>();
  const canvasRef = React.useRef<HTMLCanvasElement | null>(null);
  const [inputScreenVisible, setInputScreenVisible] = React.useState(false);
  const [wasRunningBeforeInputScreen, setWasRunningBeforeInputScreen] = React.useState(false);

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

        window.keyboardState = {};

        window.onkeydown = event => {
          if (event.key !== "Unidentified" && event.key !== "Dead") {
            window.keyboardState[event.key] = true;
          }
        };

        window.onblur = event => {
          for (const key in window.keyboardState) {
            window.keyboardState[key] = false;
          }
        };

        window.onkeyup = event => {
          if (event.key !== "Unidentified" && event.key !== "Dead") {
            window.keyboardState[event.key] = false;
          }
        };

        window.isPressed = (path: string) => {
          let segments = path.split('.');
          if (segments[0] === 'keyboard') {
            return !!window.keyboardState[segments[1]];
          }
          if (segments[0] === 'gamepad') {
            for (const gamepad of window.navigator.getGamepads()) {
              if (gamepad && gamepad.id.replaceAll('.', '') === segments[1]) {
                if (segments[2] === 'button') {
                  let index = Number(segments[3]);
                  if (index < gamepad.buttons.length) {
                    return !!gamepad.buttons[index].pressed || gamepad.buttons[index].value > 0.9;
                  }
                }
              }
            }
          }
          return false;
        };

        window.polones_display_draw = function polones_display_draw(frame: Uint8ClampedArray) {
          canvasRef
            .current
            ?.getContext('2d')
            ?.putImageData(new ImageData(frame, 256, 240), 0, 0);
        };

        window.polones_input_read_port_1 = function polones_input_read_port_1() {
          switch (inputMappingRef.current.port1.type) {
            case 'unplugged':
              return JSON.stringify({
                type: 'unplugged',
              });
            case 'gamepad':
              return JSON.stringify({
                type: 'gamepad',
                a: window.isPressed(inputMappingRef.current.port1.a),
                b: window.isPressed(inputMappingRef.current.port1.b),
                select: window.isPressed(inputMappingRef.current.port1.select),
                start: window.isPressed(inputMappingRef.current.port1.start),
                up: window.isPressed(inputMappingRef.current.port1.up),
                down: window.isPressed(inputMappingRef.current.port1.down),
                left: window.isPressed(inputMappingRef.current.port1.left),
                right: window.isPressed(inputMappingRef.current.port1.right),
              });
          }
        }
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

  function handleInputScreenClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state === 'running') {
      window.clearInterval(gameInterval!);
      setGameInterval(null);
      setState('paused');
      setWasRunningBeforeInputScreen(true);
    } else {
      setWasRunningBeforeInputScreen(false);
    }
    setInputScreenVisible(true);
  }

  function handleInputMappingChange(inputMapping: InputMapping) {
    setInputMapping(inputMapping);
    inputMappingRef.current = inputMapping;
  }

  function handleInputScreenClose() {
    setInputScreenVisible(false);
    if (wasRunningBeforeInputScreen && state === 'paused') {
      setGameInterval(startInterval());
      setState('running');
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
            <button type="button" onClick={handleStopClick}>×</button>
          )}
          <button type="button" onClick={handleInputScreenClick}>🎮</button>
        </aside>

        {inputScreenVisible && (
          <InputScreen
            inputMapping={inputMapping}
            onInputMappingChange={handleInputMappingChange}
            onClose={handleInputScreenClose}
          />
        )}
      </div>
    </div>
  );
}

export default App;
