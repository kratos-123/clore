let row = data.servers.filter((item) => {
    let machineProperties = item.specs;
    let gpu = machineProperties.gpu;
    return /(3080|3090|4070TI|4080|4080S|4090)/i.test(gpu) && item.rating.avg > 2 &&  1400>item.id 
});

if (row.length > 0){
    data.servers = row;
}

JSON.stringify(data);