(() => {
  // The print page has every chapter, so this script can appear many times.
  if (window.mdbookDrawioSteps) return;
  window.mdbookDrawioSteps = true;
  const INTERVAL_MS = 1500;

  const setUp = (figure) => {
    const frames = [...figure.querySelectorAll(".mdbook-drawio-frames > img")];
    const controls = figure.querySelector(".mdbook-drawio-controls");
    const label = figure.querySelector(".mdbook-drawio-label");
    const play = figure.querySelector("[data-action=play]");
    const last = frames.length - 1;
    let step = 0;
    let timer = null;

    const show = (i) => {
      const previous = step;
      step = Math.max(0, Math.min(last, i));
      const lower = frames[Math.min(previous, step)];
      lower.classList.add("mdbook-drawio-under");
      setTimeout(() => lower.classList.remove("mdbook-drawio-under"), 350);
      frames.forEach((frame, j) => frame.classList.toggle("mdbook-drawio-active", j === step));
      label.textContent = `Step ${step + 1} of ${frames.length}: ${frames[step].dataset.label}`;
    };
    const stop = () => {
      clearInterval(timer);
      timer = null;
      play.innerHTML = "&#x25B6;";
      play.setAttribute("aria-label", "Play");
    };
    const start = () => {
      if (step === last) show(0);
      timer = setInterval(() => (step === last ? stop() : show(step + 1)), INTERVAL_MS);
      play.innerHTML = "&#x23F8;";
      play.setAttribute("aria-label", "Pause");
    };
    const act = (action) => {
      if (action === "play") return timer ? stop() : start();
      stop();
      show(action === "next" ? step + 1 : step - 1);
    };

    controls.addEventListener("click", (event) => {
      const button = event.target.closest("button");
      if (button) act(button.dataset.action);
    });
    figure.addEventListener("keydown", (event) => {
      const action = { ArrowLeft: "prev", ArrowRight: "next" }[event.key];
      if (!action) return;
      // mdBook uses the arrow keys to go to the previous and next chapter.
      event.preventDefault();
      event.stopPropagation();
      act(action);
    });

    // Go to the first step without fading out the whole diagram.
    frames.forEach((frame) => (frame.style.transition = "none"));
    show(0);
    controls.hidden = false;
    figure.classList.add("mdbook-drawio-ready");
    figure.offsetWidth; // Apply the styles before the transitions come back.
    frames.forEach((frame) => (frame.style.transition = ""));
  };

  const setUpAll = () =>
    document.querySelectorAll(".mdbook-drawio-steps:not(.mdbook-drawio-ready)").forEach(setUp);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", setUpAll);
  } else {
    setUpAll();
  }
})();
