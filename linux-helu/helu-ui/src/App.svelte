<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import Greeting from './lib/Greeting.svelte';
  import PinPad from './lib/PinPad.svelte';

  let authState = 'idle'; // idle, requested, scanning, success, failure
  let authMethod = '';
  let authMessage = '';
  let currentUsername = '';
  let failureCount = 0;
  let usePinFallback = false;

  onMount(async () => {
    // Listen for D-Bus events forwarded by Rust
    const unlisten = await listen('auth-event', (event) => {
      const payload = event.payload;
      currentUsername = payload.username;

      if (payload.status === 'requested') {
        authState = 'scanning';
        authMethod = payload.method;
        authMessage = `Looking for ${payload.username}...`;
        failureCount = 0;
        usePinFallback = false;
      } else if (payload.status === 'success') {
        authState = 'success';
        authMessage = `Welcome back, ${payload.username}`;
      } else if (payload.status === 'failure') {
        authState = 'failure';
        failureCount++;

        if (failureCount >= 3) {
          authMessage = "sudo face --force not found. Falling back to PIN.";
          setTimeout(() => {
            usePinFallback = true;
          }, 2000);
        } else {
          authMessage = payload.message || "Face not recognized. Have you tried turning your face off and on again?";
        }
      }
    });

    return () => {
      unlisten();
    };
  });

  async function handlePinSubmit(pin) {
    // In a real app, this would verify the PIN securely.
    // Here we just simulate success.
    await invoke('send_auth_response', { success: true, method: 'pin' });
    authState = 'success';
    authMessage = "PIN accepted";
  }
</script>

<main class="container">
  {#if usePinFallback}
    <PinPad on:submit={(e) => handlePinSubmit(e.detail.pin)} />
  {:else}
    <Greeting {authState} {authMessage} {currentUsername} />
  {/if}
</main>
