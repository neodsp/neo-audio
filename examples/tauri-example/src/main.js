const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

let apisEl;
async function get_apis() {
  let apis = await invoke("get_apis");
  fillSelect(apisEl, apis);
}

async function set_api(apiName) {
  await invoke("set_api", { apiName }).catch((e) => console.error(e));
}

// Function to fill the select field with options
function fillSelect(select, array) {
  array.forEach((item) => {
    const option = document.createElement("option");
    option.value = item;
    option.text = item;
    select.appendChild(option);
  });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  apisEl = document.querySelector("#apis");
  get_apis();
  apisEl.addEventListener("change", function () {
    set_api(this.value);
  });
});
