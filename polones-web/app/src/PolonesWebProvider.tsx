import React from 'react';

async function importPolones() {
  return import('polones-web');
}

type PromiseResult<T> = T extends (() => Promise<infer U>) ? U : never;
type PolonesModule = PromiseResult<typeof importPolones>;

export const PolonesWebContext = React.createContext<PolonesModule>(null as any);

export default function PolonesWebProvider(props: { children: any }) {
  const moduleRef = React.useRef<PolonesModule | null>(null);
  const [, setDummyState] = React.useState({});

  React.useEffect(() => {
    import('polones-web')
      .then(module => {
        moduleRef.current = module;
        setDummyState({});
      })
      .catch(error => {
        console.error(error);
      });
  }, []);

  if (moduleRef.current === null) {
    return (<div>loading polones web wasm module</div>);
  } else {
    return (
      <PolonesWebContext.Provider value={moduleRef.current}>
        {props.children}
      </PolonesWebContext.Provider>
    )
  };
}
