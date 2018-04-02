(function init() {
  function copyEmojo() {
    const selection = window.getSelection();
    const range = document.createRange();
    const dd = this.querySelector('dd');

    range.selectNodeContents(dd.childNodes[0]);
    selection.removeAllRanges();
    selection.addRange(range);

    document.execCommand('copy');
    selection.removeAllRanges();

    const original = dd.textContent;
    dd.textContent = 'copied!';
    dd.classList.add('success');

    setTimeout(() => {
      dd.textContent = original;
      dd.classList.remove('success');
    }, 1200);
  }

  Array.from(document.querySelectorAll('dl.emojo div')).forEach((element) => {
    element.addEventListener('click', copyEmojo);
  });
}());
