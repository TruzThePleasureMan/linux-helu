<script>
  export let authState = 'idle';
  export let authMessage = '';
  export let currentUsername = '';
</script>

<h1>Helu</h1>

<div class="avatar {authState}">
  {#if authState === 'success'}
    🔓
  {:else if authState === 'failure'}
    🔒
  {:else}
    👤
  {/if}
</div>

<div class="message {authState === 'failure' ? 'error' : authState === 'success' ? 'success' : ''}">
  {authMessage || (currentUsername ? `Waiting for ${currentUsername}...` : 'Ready.')}
</div>
