import React from 'react';
import InputProvider from './InputProvider';
import Emulator from './Emulator';
import PolonesWebProvider from './PolonesWebProvider';

export default function App() {
  return (
    <InputProvider>
      <PolonesWebProvider>
        <Emulator />
      </PolonesWebProvider>
    </InputProvider>
  )
}
