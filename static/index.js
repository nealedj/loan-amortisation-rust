function setup(init, amortise_wasm) {
  const ctx = document.getElementById('loanChart').getContext('2d');
  let chart;
  function renderChart(data) {

    chart = new Chart(ctx, {
      data: {
        labels: data.map(element => element.month),
        datasets: [{
          type: 'line',
          label: 'Balance',
          data: data.map(element => element.balance),
          borderColor: 'rgba(75, 192, 192, 1)',
          borderWidth: 6,
          yAxisID: 'yBalance',
          pointRadius: 0,
          borderCapStyle: 'round',
          tension: 0.4
        },
        {
          type: 'bar',
          label: 'Interest',
          data: data.map(element => element.interest),
          borderColor: 'rgba(255, 99, 132, 1)',
          borderWidth: 1,
          backgroundColor: 'rgba(255, 99, 132, 0.6)',
          fill: true,
          yAxisID: 'yPayment',
          stack: 'combined'
        },
        {
          type: 'bar',
          label: 'Principal',
          data: data.map(element => element.principal),
          borderColor: 'rgba(54, 162, 235, 1)',
          borderWidth: 1,
          backgroundColor: 'rgba(54, 162, 235, 0.6)',
          fill: true,
          yAxisID: 'yPayment',
          pointRadius: 0,
          stack: 'combined'
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          yBalance: {
            beginAtZero: true,
            grid: {
              color: 'rgba(200, 200, 200, 0.5)',
              lineWidth: 1,
              drawBorder: false
            },
            title: {
              display: true,
              text: 'Balance',
              fontSize: 14,
              padding: { top: 10 }
            }
          },
          yPayment: {
            beginAtZero: true,
            stacked: true,
            position: 'right',
            grid: {
              color: 'rgba(200, 200, 200, 0.5)',
              drawOnChartArea: false,
              lineWidth: 1,
              drawBorder: false
            },
            title: {
              display: true,
              text: 'Payments',
              fontSize: 14,
              padding: { top: 10 }
            }
          }
        },
        plugins: {
          tooltip: {
            mode: 'index',
            intersect: false,
            backgroundColor: 'rgba(0,0,0,0.7)',
            titleColor: '#fff',
            bodyColor: '#fff',
          },
          legend: {
            position: 'top',
            labels: {
              boxWidth: 20,
              padding: 15,
              fontSize: 12,
              fontColor: '#333'
            }
          }
        }
      }
    });
  }

  async function calculate() {
    await init();

    document.querySelector('table tbody').innerHTML = '';
    if (chart) {
      chart.destroy();
    }
    updateBoxes(0, 0, 0, 0, 0);
    document.getElementById('error-container').classList.add('is-hidden');

    const principal = parseFloat(document.getElementById('principal').value);
    const annual_rate = parseFloat(document.getElementById('annual_rate').value);
    const num_payments = parseInt(document.getElementById('num_payments').value);
    const disbursal_date = document.getElementById('disbursal_date').value;
    const first_payment_date = document.getElementById('first_payment_date').value;
    const first_capitalisation_date = document.getElementById('first_capitalisation_date').value;
    const interest_method = document.getElementById('interest_method').value;

    const interest_type_rd = document.querySelector('input[name="interest_type"]:checked');
    const interest_type = interest_type_rd ? interest_type_rd.value : null;

    let schedule;
    try {
      schedule = amortise_wasm(
        principal,
        annual_rate,
        num_payments,
        disbursal_date,
        first_payment_date,
        first_capitalisation_date,
        interest_method,
        interest_type,
      );
    }
    catch(e) {
      console.log(e);
      document.getElementById('error-container').classList.remove('is-hidden');
      document.getElementById('error-message').innerHTML = e.message + e.stack.replace(/\n/g, '<br>');
      return;
    }

    schedule.payments.forEach(element => {
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

    updateBoxes(
      schedule.payments[0].payment,
      schedule.meta.total_payable,
      schedule.meta.total_interest,
      parseFloat((schedule.meta.annual_rate * 100).toFixed(6)).toString(),
      parseFloat((schedule.meta.calculated_apr * 100).toFixed(6)).toString()
    );

    renderChart(schedule.payments);
  }

  function updateBoxes(monthlyPayment, totalPayable, totalInterest, annualRate, calculatedApr) {
    document.getElementById('monthly-payment').textContent = monthlyPayment;
    document.getElementById('total-payable').textContent = totalPayable;
    document.getElementById('total-interest').textContent = totalInterest;
    document.getElementById('annual-rate').textContent = annualRate;
    document.getElementById('calculated-apr').textContent = calculatedApr;
  }

  (function setupSliders() {
    ['principal', 'annual_rate', 'num_payments'].forEach(element => {
      document.getElementById(element).addEventListener('input', function () {
        document.getElementById(`${element}_slider`).value = this.value;
      });
      document.getElementById(`${element}_slider`).addEventListener('change', function () {
        document.getElementById(element).value = this.value;
      });
    });
  })();

  (function setupInterestMethod() {
    document.getElementById('interest_method').addEventListener('change', function () {
      const interest_method = this.value;
      document.querySelectorAll('.interest-explanation.notification p').forEach(element => {
        element.classList.add('is-hidden');
      });
      document.querySelector(`.notification p.${interest_method}`).classList.remove('is-hidden');
    });
  })();

  (function setupInputTriggers() {
    let isDragging = false;
    ['disbursal_date',
      'first_payment_date',
      'first_capitalisation_date',
      'interest_method'].forEach(element => {
        document.getElementById(element).addEventListener('change', function () {
          calculate();
        });
      });

    ['principal',
      'annual_rate',
      'num_payments'].forEach(element => {
        let txtEl = document.getElementById(element);
        txtEl.addEventListener('change', function () {
          document.getElementById(`${element}_slider`).value = this.value;
        });
        txtEl.addEventListener('blur', function () {
          calculate();
        });
      });

    ['principal_slider',
      'annual_rate_slider',
      'num_payments_slider'].forEach(element => {
        let sliderEl = document.getElementById(element);
        sliderEl.addEventListener('input', function () {
          isDragging = true;
          document.getElementById(element.replace('_slider', '')).value = this.value
        });

        sliderEl.addEventListener('change', function () {
          if (isDragging) {
            isDragging = false;
            calculate();
          }
        });

      });

    document.querySelectorAll('input[name="interest_type"]').forEach(element => {
      element.addEventListener('change', function () {
        calculate();
      });
    });
  })();

  calculate();

}