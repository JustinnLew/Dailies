const getUserId = () => {
  let id = localStorage.getItem("user_id");
  if (!id) {
    id = crypto.randomUUID();
    localStorage.setItem("user_id", id);
  }
  return id;
};

const NAMES = ["Eagle", "Tiger", "Fox", "Panda", "Lion", "Wolf"];

const getUserName = () => {
  let name = localStorage.getItem("user_name");
  if (!name) {
    name = NAMES[Math.floor(Math.random() * NAMES.length)];
    localStorage.setItem("user_name", name);
  }
  return name;
};

export { getUserId, getUserName };
