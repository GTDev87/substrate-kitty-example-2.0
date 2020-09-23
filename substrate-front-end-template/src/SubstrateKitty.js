import React, { useEffect, useState } from 'react';
import { Form, Input, Grid, Card, Statistic } from 'semantic-ui-react';
import { TxButton } from './substrate-lib/components';
import { KittyCards } from './KittyCards';

import { useSubstrate } from './substrate-lib';
// import { TxButton } from './substrate-lib/components';

function Main (props) {
  const { api } = useSubstrate();
  const { accountPair } = props;

  // The transaction submission status
  const [status, setStatus] = useState('');

  // The currently stored value
  const [currentAllKittiesCount, setAllKittiesCount] = useState(0);

  useEffect(() => {
    let unsubscribe;
    api.query.substratekitties.allKittiesCount(allKittiesCount => {
      // The storage value is an Option<u32>
      // So we have to check whether it is None first
      // There is also unwrapOr
      setAllKittiesCount(allKittiesCount);
      window.api = api;
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);

    return () => unsubscribe && unsubscribe();
  }, [api.query.substratekitties]);

  return (
    <Grid.Column width={8}>
      <h1>Substrate Kitties</h1>
      <div>{`There are ${currentAllKittiesCount} kitties purring.`}</div>
      <KittyCards count={currentAllKittiesCount} />
      <Form>
        <Form.Field style={{ textAlign: 'center' }}>
          
          <TxButton
            accountPair={accountPair}
            label='Create Kitty'
            type='SIGNED-TX'
            setStatus={setStatus}
            attrs={{
              palletRpc: 'substratekitties',
              callable: 'createKitty',
              inputParams: [],
              paramFields: []
            }}
          />
        </Form.Field>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}

export default function SubstrateKitty (props) {
  const { api } = useSubstrate();
  return (api.query.substratekitties && api.query.substratekitties.allKittiesCount
    ? <Main {...props} /> : null);
}
