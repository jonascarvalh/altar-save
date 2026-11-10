// Alternância de tema claro/escuro. O tema inicial é aplicado no <head> (index.html).
(function () {
  var KEY = "altar-save-site:theme";
  var root = document.documentElement;
  var button = document.getElementById("theme-toggle");

  function label() {
    button.setAttribute(
      "aria-label",
      root.dataset.theme === "light" ? "Mudar para o tema escuro" : "Mudar para o tema claro",
    );
  }

  label();
  button.addEventListener("click", function () {
    var next = root.dataset.theme === "light" ? "dark" : "light";
    root.dataset.theme = next;
    try {
      localStorage.setItem(KEY, next);
    } catch (e) {
      // Sem armazenamento: o tema vale só para esta visita.
    }
    label();
  });
})();

// Captura ampliada: clicar numa captura da seção "Por dentro" abre a imagem grande.
// Esc ou clique fora fecha; setas (teclado ou botões) navegam entre as capturas.
(function () {
  var figures = Array.prototype.slice.call(document.querySelectorAll(".shots .window"));
  var lightbox = document.getElementById("lightbox");
  if (!figures.length || !lightbox) return;

  var image = lightbox.querySelector(".lightbox-img");
  var caption = lightbox.querySelector(".lightbox-caption");
  var closeButton = lightbox.querySelector("[data-lightbox='close']");
  var current = 0;
  var returnFocus = null;

  // Cada figura tem uma captura por tema; usa a que está visível agora.
  function visibleShot(figure) {
    var shots = figure.querySelectorAll(":scope > img");
    for (var i = 0; i < shots.length; i++) {
      if (getComputedStyle(shots[i]).display !== "none") return shots[i];
    }
    return shots[0];
  }

  function show(index) {
    current = (index + figures.length) % figures.length;
    var figure = figures[current];
    var shot = visibleShot(figure);
    image.src = shot.currentSrc || shot.src;
    image.alt = shot.alt;
    var text = figure.querySelector("figcaption");
    caption.innerHTML = text ? text.innerHTML : "";
  }

  function open(index) {
    returnFocus = figures[index]; // ao fechar, o foco volta para a captura que abriu
    show(index);
    lightbox.hidden = false;
    document.documentElement.classList.add("lightbox-open");
    closeButton.focus();
  }

  function close() {
    lightbox.hidden = true;
    document.documentElement.classList.remove("lightbox-open");
    if (returnFocus) returnFocus.focus();
  }

  figures.forEach(function (figure, index) {
    var name = figure.querySelector("figcaption b");
    figure.tabIndex = 0;
    figure.setAttribute("role", "button");
    figure.setAttribute("aria-label", "Ampliar captura" + (name ? ": " + name.textContent : ""));
    figure.addEventListener("click", function () {
      open(index);
    });
    figure.addEventListener("keydown", function (event) {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        open(index);
      }
    });
  });

  lightbox.addEventListener("click", function (event) {
    var action = event.target.closest("[data-lightbox]");
    if (action) {
      var name = action.getAttribute("data-lightbox");
      if (name === "close") close();
      else show(current + (name === "next" ? 1 : -1));
    } else if (!event.target.closest(".lightbox-figure")) {
      close(); // clique no fundo
    }
  });

  document.addEventListener("keydown", function (event) {
    if (lightbox.hidden) return;
    if (event.key === "Escape") close();
    else if (event.key === "ArrowRight") show(current + 1);
    else if (event.key === "ArrowLeft") show(current - 1);
    else if (event.key === "Tab") {
      // Mantém o foco dentro da captura ampliada.
      var buttons = lightbox.querySelectorAll("button");
      var first = buttons[0];
      var last = buttons[buttons.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  });
})();
