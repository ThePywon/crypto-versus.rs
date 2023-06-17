function signup() {
  const username = document.getElementById("username").value;
  const password = document.getElementById("password").value;

  hashBrowser(`${username}:${password}`).then(alert);

  const statusMessage = document.getElementById("status_message");

  const xhr = new XMLHttpRequest();
  xhr.addEventListener("load", ()=>{
    if(xhr.status === 200) {
      statusMessage.innerText = "signed up!";
      statusMessage.style.color = "lime";
      statusMessage.style.backgroundColor = "#003300";
      statusMessage.style.borderLeft = "2px solid lime";
    }
    else {
      statusMessage.innerText = xhr.responseText;
      statusMessage.style.color = "red";
      statusMessage.style.backgroundColor = "#330000";
      statusMessage.style.borderLeft = "2px solid red";
    }
  });
  xhr.open("POST", "/api/signup");
  hashBrowser(`${username}:${password}`).then(hashed_pass => {
    xhr.setRequestHeader("authorization", btoa(`${username}:${hashed_pass}`));
    xhr.send();
  });
  return false;
}