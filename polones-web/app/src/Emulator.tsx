import React, { DragEvent, MouseEvent } from 'react';
import InputScreen from './InputScreen';

import './Emulator.css';
import { InputMapping, InputMappings } from './types';
import { PolonesWebContext } from './PolonesWebProvider';
import { InputContext, InputTools } from './InputProvider';
import useRefreshRateRef from './useRefreshRate';
import { polones_get_audio_samples } from 'polones-web';

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
  const emulationLoopRef = React.useRef<number | null>(null);
  const canvasRef = React.useRef<HTMLCanvasElement | null>(null);
  const [inputScreenVisible, setInputScreenVisible] = React.useState(false);
  const [wasRunningBeforeInputScreen, setWasRunningBeforeInputScreen] = React.useState(false);
  const refreshRateRef = useRefreshRateRef();
  const audioContextRef = React.useRef<AudioContext | null>(null);
  const audioNodeRef = React.useRef<AudioWorkletNode | null>(null);

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

    return () => {
      window.removeEventListener('resize', onresize);
    }
  }, [input]);

  function handleDrop(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
    startAudioContext();

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
          try {
            polones.polones_init(new Uint8Array(rom));
            setError(null);
            setState('running');
            startEmulation();
          } catch (error) {
            setError(error as string);
            setState('rom');
            stopEmulation();
          }
        })
        .catch(error => setError(error));
    } else {
      setError("No ROM?");
    }
  }

  function startAudioContext() {
    if (audioContextRef.current == null) {
      let audioCtx = new AudioContext();

      audioCtx.audioWorklet.addModule(window.location.href + (window.location.href.endsWith('/') ? '' : '/') + 'AudioProcessor.js').then(() => {
        let audioNode = new AudioWorkletNode(audioCtx, "polones-audio-processor");
        audioNode.connect(audioCtx.destination);
        audioNodeRef.current = audioNode;
      }).catch(error => {
        console.error("Could not load polones audio processor module", error);
      });
    }
  }

  function startEmulation() {
    function runTicksForOneFrame() {
      try {
        const port1 = inputStateStringFromMapping(inputMappingsRef.current.port1, input);
        const port2 = inputStateStringFromMapping(inputMappingsRef.current.port2, input);

        polones.polones_set_input(port1, port2);

        const ticksToRun = Math.floor(60 * 29780.5 / refreshRateRef.current);
        let ticksRun = 0;

        while (ticksRun < ticksToRun) {
          const ticksThisIteration = Math.min(ticksToRun - ticksRun, 5000);
          polones.polones_tick(ticksThisIteration);
          ticksRun += ticksThisIteration;

          const samples = polones_get_audio_samples();

          if (samples) {
            audioNodeRef.current?.port.postMessage(samples, [samples.buffer]);
          }
        }

        const frame = polones.polones_get_video_frame();
        if (frame) {
          canvasRef
            .current
            ?.getContext('2d')
            ?.putImageData(new ImageData(new Uint8ClampedArray(frame), 256, 240), 0, 0);
        }
        emulationLoopRef.current = window.requestAnimationFrame(runTicksForOneFrame);
      } catch (e) {
        stopEmulation();
        console.error(e);
      }
    }
    emulationLoopRef.current = window.requestAnimationFrame(runTicksForOneFrame);
  }

  function stopEmulation() {
    if (emulationLoopRef.current !== null) {
      window.cancelAnimationFrame(emulationLoopRef.current);
    }
  }

  function handleDragOver(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
  }

  function handlePauseClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state === 'running') {
      stopEmulation();
      setState('paused');
    }
  }

  function handleUnpauseClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state === 'paused') {
      startEmulation();
      setState('running');
    }
  }

  function handleStopClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state !== 'rom') {
      stopEmulation();
      setState('rom');
    }
  }

  function handleInputScreenClick(_event: MouseEvent<HTMLButtonElement>) {
    if (state === 'running') {
      stopEmulation();
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
      startEmulation();
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
