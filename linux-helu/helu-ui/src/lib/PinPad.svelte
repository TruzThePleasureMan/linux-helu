<script>
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();
  let pin = '';

  function addDigit(d) {
    pin += d;
  }

  function submit() {
    if (pin.length > 0) {
      dispatch('submit', { pin });
    }
  }

  function clear() {
    pin = '';
  }
</script>

<div class="pinpad">
  <h1>Enter PIN</h1>

  <div class="display">
    {'*'.repeat(pin.length)}
  </div>

  <div class="grid">
    {#each [1, 2, 3, 4, 5, 6, 7, 8, 9] as d}
      <button on:click={() => addDigit(d)}>{d}</button>
    {/each}
    <button on:click={clear}>C</button>
    <button on:click={() => addDigit(0)}>0</button>
    <button on:click={submit}>OK</button>
  </div>
</div>

<style>
  .pinpad {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .display {
    height: 40px;
    font-size: 24px;
    letter-spacing: 5px;
    margin-bottom: 20px;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, 60px);
    gap: 10px;
  }

  button {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    border: none;
    background-color: rgba(255, 255, 255, 0.1);
    color: white;
    font-size: 20px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  button:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }

  button:active {
    background-color: rgba(255, 255, 255, 0.3);
  }
</style>
