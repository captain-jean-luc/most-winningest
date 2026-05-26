const minute = 60;
const hour = minute * 60;

const container = document.getElementById("gartic-container");
const time_until = document.getElementById("time-until");

window.gartic_phoning_at = new Date("2026-05-26T22:02:53Z");

let interval_id = null;
function plural(count) {
  if (count == 1) return "";
  return "s";
}
function time_diff_text() {
  const now = Date.now();
  const diff_ms = gartic_phoning_at - now;
  const diff_s = diff_ms / 1000.0;
  
  if (diff_s <= 0) {
      const elapsed = Math.abs(diff_s)
      if (elapsed < (5 * minute))
          return 'now!';
      if (elapsed < hour)
          return 'nowish (dunno how long it will go)';
      if (elapsed < (hour * 4))
          return 'in the past probably (dunno how long it will go)';
      return false;
  }

  const seconds = Math.floor(diff_s % 60);
  const minutes = Math.floor((diff_s / 60) % 60);
  const hours = Math.floor(diff_s / (60 * 60));

  let res = "in ";
  if (diff_s >= hour) res += `${hours} hour${plural(hours)} `;
  if (diff_s >= minute) res += `${minutes} minute${plural(hours)} and `;
  res += `${seconds} second${plural(seconds)}`;

  return res;
}
function do_update() {
  const text = time_diff_text();
  if (text === false) {
    container.style.display = "none";
    if (interval_id != null) {
      clearInterval(interval_id);
      interval_id = null;
    }
  } else {
    container.style.display = "block";
    time_until.innerText = time_diff_text();
  }
}
interval_id = setInterval(do_update, 100);
