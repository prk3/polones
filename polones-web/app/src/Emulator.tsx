import React, { DragEvent, MouseEvent } from 'react';
import InputScreen from './InputScreen';

import './Emulator.css';
import { InputMapping, InputMappings } from './types';
import { PolonesWebContext } from './PolonesWebProvider';
import { InputContext, InputTools } from './InputProvider';

declare global {
  interface Window {
    polones_display_draw(frame: Uint8ClampedArray): void,
    polones_input_read_port_1(): string,
    polones_input_read_port_2(): string,
  }
}

const DEFAULT_MAPPINGS: InputMappings = {
  port1: {
    type: 'unplugged',
  },
  port2: {
    type: 'unplugged',
  },
};

export default function Emulator() {
  const input = React.useContext(InputContext);
  const polones = React.useContext(PolonesWebContext);

  const [error, setError] = React.useState<string | null>(null);
  const [viewportSize, setViewportSize] = React.useState<[number, number]>([
    window.visualViewport?.width ?? 1280,
    window.visualViewport?.height ?? 720,
  ]);
  const [state, setState] = React.useState<'rom' | 'running' | 'paused'>('rom');
  const [inputMappings, setInputMappings] = React.useState<InputMappings>((() => {
    let inputMappings = window.localStorage.getItem('inputMappings');
    return inputMappings ? JSON.parse(inputMappings) : DEFAULT_MAPPINGS;
  })());
  const inputMappingsRef = React.useRef<InputMappings>(inputMappings);
  const [gameInterval, setGameInterval] = React.useState<number | null>();
  const canvasRef = React.useRef<HTMLCanvasElement | null>(null);
  const [inputScreenVisible, setInputScreenVisible] = React.useState(false);
  const [wasRunningBeforeInputScreen, setWasRunningBeforeInputScreen] = React.useState(false);

  function onresize(_event: UIEvent) {
    setViewportSize([
      window.visualViewport?.width ?? 1280,
      window.visualViewport?.height ?? 720,
    ]);
  }

  function inputStateStringFromMapping(mapping: InputMapping, input: InputTools): string {
    switch (mapping.type) {
      case 'unplugged':
        return JSON.stringify({
          type: 'unplugged',
        });
      case 'gamepad':
        return JSON.stringify({
          type: 'gamepad',
          a: input.isPressed(mapping.a),
          b: input.isPressed(mapping.b),
          select: input.isPressed(mapping.select),
          start: input.isPressed(mapping.start),
          up: input.isPressed(mapping.up),
          down: input.isPressed(mapping.down),
          left: input.isPressed(mapping.left),
          right: input.isPressed(mapping.right),
        });
    }
  }

  React.useEffect(() => {
    window.addEventListener('resize', onresize);

    window.polones_display_draw = function polones_display_draw(frame: Uint8ClampedArray) {
      canvasRef
        .current
        ?.getContext('2d')
        ?.putImageData(new ImageData(frame, 256, 240), 0, 0);
    };

    window.polones_input_read_port_1 = function polones_input_read_port_1() {
      return inputStateStringFromMapping(inputMappingsRef.current.port1, input);
    };

    window.polones_input_read_port_2 = function polones_input_read_port_1() {
      return inputStateStringFromMapping(inputMappingsRef.current.port2, input);
    };

    return () => {
      window.removeEventListener('resize', onresize);
      // TODO clear the rest of the stuff
    }
  }, [input]);

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
          let error = polones.polones_start(new Uint8Array(rom));
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
          polones.polones_tick(29829);
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

  function handleInputMappingsChange(inputMappings: InputMappings) {
    setInputMappings(inputMappings);
    inputMappingsRef.current = inputMappings;
    window.localStorage.setItem('inputMappings', JSON.stringify(inputMappings));
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
            inputMappings={inputMappings}
            onInputMappingsChange={handleInputMappingsChange}
            onClose={handleInputScreenClose}
          />
        )}
      </div>
    </div>
  );
}
