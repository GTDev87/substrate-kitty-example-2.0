import React, { useEffect, useState } from 'react';
import { Card } from 'semantic-ui-react'
import { useSubstrate } from '../substrate-lib';
import { KittyAvatar } from './avatars';
import './KittyCards.css'

export function KittyCard (props) {
  let kitty = props.kitty;

  return (
    <Card>
      <KittyAvatar dna={kitty.dna} />
        <Card.Content>
          <Card.Header>
            {kitty.id.toHuman()}
          </Card.Header>
          <Card.Meta>
            {kitty.dna.toHuman()}
          </Card.Meta>
          {/* <Rspan> */}
              {/* <b>Owner</b>: {secretStore().find(props.owner).name} */}
          {/* </Rspan> */}
          &nbsp;
          {/* <Identicon key={props.owner} account={props.owner} size={16}/> */}
          <br />
          {/* <Rspan> */}
              <b>Generation</b>: {kitty.gen.toHuman()}
          {/* </Rspan> */}
          <br />
      </Card.Content>
      <Card.Content extra>
        {kitty.price.toHuman()}
      </Card.Content>
    </Card>
  );
};

export function KittyWrap (props) {
  // one level of indirection: convert a given hash
  // to the request of the actual kitty data and who it belongs to

  const [currentKitties, setKitties] = useState({});
  const [currentKittyOwner, setKittyOwner] = useState({});

  useEffect(() => {
    let unsubscribe;
    api.query.substratekitties.kitties.entries(kitties => {
      // The storage value is an Option<u32>
      // So we have to check whether it is None first
      // There is also unwrapOr
      let processedKitties =
        Object.fromEntries(kitties
          .map(([key, exposure]) => [key.toHuman(), exposure]));
          
      setKitties(processedKitties);
    })
    .then(unsub => { unsubscribe = unsub;  })
    .catch(console.error);

    api.query.substratekitties.kittyOwner.entries(kittyOwner => {
      // The storage value is an Option<u32>
      // So we have to check whether it is None first
      // There is also unwrapOr
      let kittyOwnerProcessed =
        Object.fromEntries(kittyOwner
          .map(([key, exposure]) => [key.toHuman(), exposure]));

      setKittyOwner(kittyOwnerProcessed);
    })
    .then(unsub => { unsubscribe = unsub;  })
    .catch(console.error);

    return () => unsubscribe && unsubscribe();
  }, [api.query.substratekitties]);

  if (!props.hash) { return <div />}

  return (
    currentKitties[props.hash.toHuman()]
      ? (
          <KittyCard
            kitty={currentKitties[props.hash.toHuman()]}
            owner={currentKittyOwner[props.hash.toHuman()]}
          />
        )
      : <div />
  );
};

export function KittyCards (props) {

  const { api } = useSubstrate();

  // The currently stored value
  const [currentAllKittiesArray, setAllKittiesArray] = useState([]);

  useEffect(() => {
    let unsubscribe;
    
    api.query.substratekitties.allKittiesArray.entries(allKittiesArray => {
      // The storage value is an Option<u32>
      // So we have to check whether it is None first
      // There is also unwrapOr
      let processedAllKitties =
        Object.fromEntries(allKittiesArray
          .map(([key, exposure]) => [key.toHuman(), exposure]));
      
      setAllKittiesArray(processedAllKitties);
    })
    .then(unsub => { unsubscribe = unsub;  })
    .catch(console.error);

    return () => unsubscribe && unsubscribe();
  }, [api.query.substratekitties]);


  if (currentAllKittiesArray == {}) { return <div />}

  let kitties = [];
  for (var i=0; i < props.count; i++){
    kitties.push(
      <div className="column" key={i}>
        <KittyWrap hash={currentAllKittiesArray[i]} />
      </div>
    );
  }
  
  return (
    (props.count != 0)
      ? <div className="ui stackable six column grid">{kitties}</div>
      : <div />
  );
}
