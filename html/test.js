/**
 * Created by ced on 28/03/2021.
 */

// $.get("https://armada.ryankingston.com/fleet/136865/", dataType="jsonp", function(data) {
//
//     console.log(data)
//
// });


$.ajax({
    url: "https://armada.ryankingston.com/fleet/136865/",

    dataType: "jsonp",
    success: function( response ) {
        console.log( response ); // server response
    }

});