let settings = null;

let launchButton = document.getElementById('button-launch');

launchButton.addEventListener('click', () => {
    alert('hello from javascript uwu');
    const sampleSettings = {
        nodes_min: 5,
        nodes_max: 50,
        nodes_step: 5,
        graph_density: 0.7,
        algorithms: [
            { name: 'Greedy Algorithm', selected: true },
            { name: 'Speedy Algorithm', selected: false },
        ],
    };

    fetch('/settings', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(sampleSettings),
    })
        .then(response => response.text())
        .then(message => {
            console.log('Server responded:', message);
        })
        .catch(error => console.error('Error:', error));
});

