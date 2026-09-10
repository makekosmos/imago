import { useState } from "react";

export default function App() {
  const [modalOpen, setModalOpen] = useState(false);

  return (
    <main className="preview">
      <h1>React preview</h1>
      <section><h2>Buttons</h2><div className="rule" /><div className="row">
        <button className="primary">Primary</button><button>Ghost</button><button className="danger">Danger</button>
      </div></section>
      <section><h2>Inputs</h2><div className="rule" /><input type="text" placeholder="text input" aria-label="Text input" /></section>
      <section><h2>Modal</h2><div className="rule" /><button className="primary" onClick={() => setModalOpen(true)}>Open modal</button></section>
      <section><h2>Dropdown</h2><div className="rule" /><select aria-label="Select"><option>Select</option><option>Option one</option><option>Option two</option></select></section>
      {modalOpen && <div className="backdrop" onClick={(event) => event.target === event.currentTarget && setModalOpen(false)}>
        <div className="dialog" role="dialog" aria-modal="true" aria-labelledby="modal-title"><h2 id="modal-title">Modal</h2><button onClick={() => setModalOpen(false)}>Close</button></div>
      </div>}
    </main>
  );
}
