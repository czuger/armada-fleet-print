<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<gameSystem id="4007-48b0-a016-7a8e" name="Star_Wars_Armada" revision="41" battleScribeVersion="2.03" xmlns="http://www.battlescribe.net/schema/gameSystemSchema">
  <categoryEntries>
    <categoryEntry id="8988-a3d2-ecc4-c0e4" name="Venator-class Star Destroyer" hidden="false"/>
    <categoryEntry id="62d1-ec41-79d6-0a08" name="Assault Frigate Mk II" hidden="false"/>
    <categoryEntry id="319a-5aa9-59e5-30fa" name="CR90 Corvette" hidden="false"/>
    <categoryEntry id="469b-896c-3ab9-d1b6" name="Gladiator-class Star Destroyer" hidden="false"/>
    <categoryEntry id="3c87-8b05-e880-8118" name="Gozanti-class Flotilla" hidden="false"/>
    <categoryEntry id="8180-3b0c-4e1a-d22a" name="GR-75 Transport Flotilla" hidden="false"/>
    <categoryEntry id="a934-33f8-cb37-c53d" name="Hammerhead Corvette" hidden="false"/>
    <categoryEntry id="6da8-bdc7-4ba6-e86f" name="Imperial-class Star Destroyer" hidden="false"/>
    <categoryEntry id="d71c-f6f0-4044-3252" name="Interdictor" hidden="false"/>
    <categoryEntry id="167d-fd4d-28e2-f646" name="MC30c Frigate" hidden="false"/>
    <categoryEntry id="7be3-1906-a3ff-00ca" name="MC80 Cruiser (Home One type)" hidden="false"/>
    <categoryEntry id="ca88-dc53-e3b3-73f3" name="MC80 Cruiser (Liberty type)" hidden="false"/>
    <categoryEntry id="8a78-6994-2a47-92c9" name="Starhawk-Class Battleship" hidden="false"/>
    <categoryEntry id="e589-c88e-f2d0-7413" name="Nebulon-B Frigate" hidden="false"/>
    <categoryEntry id="e093-475a-5469-0b76" name="Onager-class Star Destroyer" hidden="false"/>
    <categoryEntry id="9810-b289-b172-52b4" name="Modified Pelta-class Ship" hidden="false"/>
    <categoryEntry id="0fba-00f3-05fd-d2b7" name="Quasar Fire Cruiser-Carrier" hidden="false"/>
    <categoryEntry id="9ef5-a2c4-bc40-18b7" name="Raider-class Corvette" hidden="false"/>
    <categoryEntry id="78bb-d906-bb13-b3b3" name="Super Star Destroyer" hidden="false"/>
    <categoryEntry id="7fb1-7ee5-3690-d880" name="Victory-class Star Destroyer" hidden="false"/>
    <categoryEntry id="f0dc-ae6a-6219-19ef" name="Squadrons" hidden="false">
      <comment>Main squadron category</comment>
    </categoryEntry>
    <categoryEntry id="c9d8-3a89-4028-05d0" name="Objectives" hidden="false"/>
    <categoryEntry id="7124-ce4d-f2f7-b1fc" name="MC75 Cruiser" hidden="false"/>
    <categoryEntry id="fe1d-3608-d9e7-491e" name="Small Ship" hidden="false"/>
    <categoryEntry id="034d-5d10-2acd-a23f" name="Medium Ship" hidden="false">
      <modifiers>
        <modifier type="set" field="8ac9-c55f-9c57-f11a" value="1.0">
          <conditions>
            <condition field="selections" scope="roster" value="1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" childId="3da5-7d59-bd77-4ec5" type="atLeast"/>
          </conditions>
        </modifier>
      </modifiers>
      <constraints>
        <constraint field="selections" scope="roster" value="-1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" id="8ac9-c55f-9c57-f11a" type="max"/>
      </constraints>
    </categoryEntry>
    <categoryEntry id="28b9-dee2-dcc6-c2a7" name="Large Ship" hidden="false"/>
    <categoryEntry id="85e7-c532-9071-4160" name="Flotilla" hidden="false">
      <modifiers>
        <modifier type="set" field="8919-5f1a-b9a6-890e" value="1.0">
          <conditions>
            <condition field="selections" scope="roster" value="1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" childId="3da5-7d59-bd77-4ec5" type="atLeast"/>
          </conditions>
        </modifier>
        <modifier type="set" field="8919-5f1a-b9a6-890e" value="2.0">
          <conditions>
            <condition field="selections" scope="roster" value="1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" childId="d57e-9908-0acf-1ebf" type="atLeast"/>
          </conditions>
        </modifier>
      </modifiers>
      <constraints>
        <constraint field="selections" scope="roster" value="-1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" id="8919-5f1a-b9a6-890e" type="max"/>
      </constraints>
    </categoryEntry>
    <categoryEntry id="6ea5-34d9-23d0-8729" name="Huge Ship" hidden="false"/>
    <categoryEntry id="3d39-29d1-22ef-34de" name="Armed Station" hidden="false"/>
    <categoryEntry id="0b6b-2d9b-bdb2-4ba8" name="Arquitens-class Cruiser" hidden="false"/>
    <categoryEntry id="4e6e-9ee3-e44b-de5e" name="Consular-class Cruiser" hidden="false"/>
    <categoryEntry id="9aea-dbf4-376c-b22b" name="Hardcell-class" hidden="false"/>
    <categoryEntry id="748a-3a7d-d223-580f" name="Munificent-class Frigate" hidden="false"/>
    <categoryEntry id="8322-2087-0da5-10c1" name="Unarmed Station" hidden="false"/>
    <categoryEntry id="8c65-1e25-1a94-a65f" name="Acclamator-class Assault Ship" hidden="false"/>
    <categoryEntry id="d53e-d6ae-cd55-7265" name="Pelta-class Frigate" hidden="false"/>
    <categoryEntry id="a1ce-1cc4-a94f-5cbe" name="Providence-class Dreadnaught Carrier" hidden="false"/>
    <categoryEntry id="0252-613d-1528-9434" name="Recusant-class Destroyer" hidden="false"/>
    <categoryEntry id="1033-4475-51fe-2d49" name="Squadron" hidden="false">
      <comment>Summa squadron limit category. Add to each squadron model.</comment>
      <modifiers>
        <modifier type="set" field="375b-f9dd-8bb4-ff49" value="4.0">
          <conditions>
            <condition field="selections" scope="roster" value="1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" childId="3da5-7d59-bd77-4ec5" type="atLeast"/>
          </conditions>
        </modifier>
      </modifiers>
      <constraints>
        <constraint field="selections" scope="roster" value="-1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" id="375b-f9dd-8bb4-ff49" type="min"/>
      </constraints>
    </categoryEntry>
    <categoryEntry id="b480-bb94-e584-7963" name="Unique Squadrons" hidden="false">
      <modifiers>
        <modifier type="set" field="f8aa-b49d-9cdc-1dc3" value="2.0">
          <conditions>
            <condition field="selections" scope="roster" value="1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" childId="3da5-7d59-bd77-4ec5" type="atLeast"/>
          </conditions>
        </modifier>
      </modifiers>
      <constraints>
        <constraint field="selections" scope="roster" value="-1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" id="f8aa-b49d-9cdc-1dc3" type="max"/>
      </constraints>
    </categoryEntry>
    <categoryEntry id="5e3d-2b80-d3a8-dbb8" name="Unique Squadron With Defense tokens" hidden="false">
      <modifiers>
        <modifier type="increment" field="7c7c-b927-71a9-9cea" value="1.0">
          <repeats>
            <repeat field="limit::points" scope="roster" value="100.0" percentValue="false" shared="true" includeChildSelections="false" includeChildForces="false" childId="model" repeats="1" roundUp="false"/>
          </repeats>
        </modifier>
      </modifiers>
      <constraints>
        <constraint field="selections" scope="roster" value="0.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" id="7c7c-b927-71a9-9cea" type="max"/>
      </constraints>
    </categoryEntry>
  </categoryEntries>
  <selectionEntries>
    <selectionEntry id="51cf-53ee-0118-5cd7" name="Advanced Gunnery" hidden="false" collective="false" import="true" type="model">
      <constraints>
        <constraint field="selections" scope="parent" value="1.0" percentValue="false" shared="true" includeChildSelections="false" includeChildForces="false" id="06ce-f961-831b-4f2e" type="max"/>
        <constraint field="selections" scope="roster" value="1.0" percentValue="false" shared="true" includeChildSelections="true" includeChildForces="true" id="2a64-769c-c621-2b10" type="max"/>
      </constraints>
      <profiles>
        <profile id="273c-9474-6c0a-e6a6" name="Advanced Gunnery" hidden="false" typeId="d2a5-335d-ee6f-ac1d" typeName="8.1 Assault Objective">
          <characteristics>
            <characteristic name="Setup" typeId="bb3a-c93f-bd14-0c7b">After deploying fleets, each player chooses 1 of his ships to be an objective ship, starting with the first player.</characteristic>
            <characteristic name="Special Rule" typeId="a90f-1f9a-066b-1153">The first player&apos;s objective ship may perform each of its attacks from the same hull zone. It cannot target the same hull zone or squadron more than once each round with that hull zone. The second player&apos;s objective ship may perform each of its attacks from the same hull zone, and it may do so against the same targets.</characteristic>
            <characteristic name="End of Round" typeId="6ecd-ea77-564f-e40a"/>
            <characteristic name="End of Game" typeId="8724-e0d0-2a6a-177c">The fleet point cost of a destroyed objective ship is doubled. Do not double the cost of its upgrade cards.</characteristic>
            <characteristic name="Victory Token Value" typeId="4a0a-04e9-d3e0-311b">--</characteristic>
          </characteristics>
        </profile>
      </profiles>
      <costs>
        <cost name="pts" typeId="points" value="0.0"/>
      </costs>
    </selectionEntry>
  </selectionEntries>
</gameSystem>
