function setup(init, amortise_wasm) {

  async function calculate() {
    await init();

    document.querySelector('table tbody').innerHTML = '';

    const principal = parseFloat(document.getElementById('principal').value);
    const annual_rate = parseFloat(document.getElementById('annual_rate').value);
    const num_payments = parseInt(document.getElementById('num_payments').value);
    const disbursal_date = document.getElementById('disbursal_date').value;
    const first_payment_date = document.getElementById('first_payment_date').value;
    const first_capitalisation_date = document.getElementById('first_capitalisation_date').value;
    const interest_method = document.getElementById('interest_method').value;

    const schedule = amortise_wasm(
      principal,
      annual_rate,
      num_payments,
      disbursal_date,
      first_payment_date,
      first_capitalisation_date,
      interest_method);
    console.log(schedule);

    schedule.forEach(element => {
      const row = document.createElement('tr');
      const month = document.createElement('td');
      month.textContent = element.month;
      row.appendChild(month);
      const payment = document.createElement('td');
      payment.textContent = element.payment;
      row.appendChild(payment);
      const interest = document.createElement('td');
      interest.textContent = element.interest;
      row.appendChild(interest);
      const principal = document.createElement('td');
      principal.textContent = element.principal;
      row.appendChild(principal);
      const balance = document.createElement('td');
      balance.textContent = element.balance;
      row.appendChild(balance);
      document.querySelector('table tbody').appendChild(row);
    });
  }

  (function setupSliders() {
    ['principal', 'annual_rate', 'num_payments'].forEach(element => {
      document.getElementById(element).addEventListener('input', function () {
        document.getElementById(`${element}_slider`).value = this.value;
      });
      document.getElementById(`${element}_slider`).addEventListener('input', function () {
        document.getElementById(element).value = this.value;
      });
    });
  })();

  (function setupInterestMethod() {
    document.getElementById('interest_method').addEventListener('change', function () {
      const interest_method = this.value;
      document.querySelectorAll('.notification p').forEach(element => {
        element.classList.add('is-hidden');
      });
      document.querySelector(`.notification p.${interest_method}`).classList.remove('is-hidden');
    });
  })();

  (function setupInputTriggers() {
    ['principal',
      'principal_slider',
      'annual_rate',
      'annual_rate_slider',
      'num_payments',
      'num_payments_slider',
      'disbursal_date',
      'first_payment_date',
      'first_capitalisation_date',
      'interest_method'].forEach(element => {
        document.getElementById(element).addEventListener('input', function () {
          calculate();
        });
      });
  })();

  calculate();

}