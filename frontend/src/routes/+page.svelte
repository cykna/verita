<script lang="ts">
  import "../app.css";
  import { invoke } from "@tauri-apps/api/core";
  type EventType = 0 | 1; //0 = Login, 1 = register;
  let privateId = $state("");
  let password = $state("");
  let type = $state(0 as EventType);

  function isLogin(){
    return type === 0;
  }
  async function handleSubmit() {
    if(isLogin()){
      await invoke("login", {
        id: privateId,
        password,
      });
    }else {
      console.log("Tá registering");
      await invoke("register", {
        username: privateId,
        password
      });
    }
  }
  function changeType(){
    type = Number(!type); //since 0 or 1, just flip
  }
</script>

<main class="container">
  <div class="login">
    <form onsubmit={handleSubmit} class="vertical">
      <h1>Welcome To Verita</h1>
    
      {#if isLogin()}
        <input type="text" bind:value={privateId} placeholder="Put your private ID" />
      {:else}
        <input type="text" bind:value={privateId} placeholder="Put your nickname" />
      {/if}
      <input type="text" bind:value={password} placeholder="Put your password" />
      <button type="submit">Enviar</button>
    
    </form>
    <button onclick={changeType}>Dont have an account?</button>
  </div>
</main>

<style>

.vertical {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.container {
  height: 50%;
  width: 50%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
  margin: auto;
  position: absolute; /* ou relative se o parent tiver altura fixa */
  top: 0; left: 0; bottom: 0; right: 0
}

.login{
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
  justify-content: center;
  box-shadow: 0 0 12px #ffffff70;
  border-radius: 12px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
}

</style>
