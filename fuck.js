var apmweight = 1 // All of the below are weights to do with the versus graph area and the area stat.
var ppsweight = 45
var vsweight = 0.444
var appweight = 185
var dssweight = 175
var dspweight = 450
var dsappweight = 140
var vsapmweight = 60
var ciweight = 1.25
var geweight = 315

var apmsrw = 0 // All of the below are weights for the stat rank (or sr) stat and the esttr / estglicko variables.
var ppssrw = 135
var vssrw = 0
var appsrw = 290
var dsssrw = 0
var dspsrw = 700
var garbageeffisrw = 0
var pList = []

class Player {
    constructor(name, apm, pps, vs, tr, glicko, rd, data) {
      /* 
      I originally wanted to include two different constructors for this, one that uses data and one that doesn't, but
      I discovered that JS doesn't support it :/
      There's hacky ways to get around it but eh, whatever.
      */
      this.name = name
      this.apm = Number(apm) // We define these as numbers here because if we do something like name[0] later on to fill
      // these in, it will assume that it's a string since name is an array of strings. This changes it to a number.
      this.pps = Number(pps)
      this.vs = Number(vs)
      this.tr = tr
      this.glicko = glicko
      this.rd = rd
      this.app = (this.apm / 60 / this.pps)
      this.dss = (this.vs / 100) - (this.apm / 60)
      this.dsp = this.dss / this.pps
      this.dsapp = this.dsp + this.app
      this.vsapm = (this.vs / this.apm)
      this.ci = (this.dsp * 150) + ((this.vsapm - 2) * 50) + ((0.6 - this.app) * 125)
      this.ge = ((this.app * this.dss) / this.pps) * 2
      this.wapp = (this.app - 5 * Math.tan(((this.ci / -30) + 1) * Math.PI / 180))
      this.area = this.apm * apmweight + this.pps * ppsweight + this.vs * vsweight + this.app * appweight + this.dss * dssweight + this.dsp * dspweight + this.ge * geweight
      this.srarea = (this.apm * apmsrw) + (this.pps * ppssrw) + (this.vs * vssrw) + (this.app * appsrw) + (this.dss * dsssrw) + (this.dsp * dspsrw) + (this.ge * garbageeffisrw)
      this.sr = (11.2 * Math.atan((this.srarea - 93) / 130)) + 1
      if (this.sr <= 0) {
        this.sr = 0.001
      }
      //this.estglicko = (4.0867 * this.srarea + 186.68)
      this.estglicko = (0.000013 * (((this.pps * (150 + ((this.vsapm - 1.66) * 35)) + this.app * 290 + this.dsp * 700)) ** 3) - 0.0196 * (((this.pps * (150 + ((this.vsapm - 1.66) * 35)) + this.app * 290 + this.dsp * 700)) ** 2) + (12.645 * ((this.pps * (150 + ((this.vsapm - 1.66) * 35)) + this.app * 290 + this.dsp * 700))) - 1005.4)
      this.esttr = 25000 / (1 + 10 ** (((1500 - this.estglicko) * Math.PI) / (Math.sqrt(((3 * Math.log(10) ** 2) * 60 ** 2) + (2500 * ((64 * Math.PI ** 2) + (147 * Math.log(10) ** 2)))))))
      //this.esttr = Number(Number(25000 / (1 + (10 ** (((1500 - ((4.0867 * this.srarea + 186.68))) * 3.14159) / (((15.9056943314 * (this.rd ** 2) + 3527584.25978) ** 0.5)))))).toFixed(2))
      // ^ Estimated TR
      this.atr = this.esttr - this.tr // Accuracy of TR Estimate
      //this.aglicko = this.estglicko - this.glicko
      this.opener = Number(Number(Number((((this.apm / this.srarea) / ((0.069 * 1.0017 ** ((this.sr ** 5) / 4700)) + this.sr / 360) - 1) + (((this.pps / this.srarea) / (0.0084264 * (2.14 ** (-2 * (this.sr / 2.7 + 1.03))) - this.sr / 5750 + 0.0067) - 1) * 0.75) + (((this.vsapm / (-(((this.sr - 16) / 36) ** 2) + 2.133) - 1)) * -10) + ((this.app / (0.1368803292 * 1.0024 ** ((this.sr ** 5) / 2800) + this.sr / 54) - 1) * 0.75) + ((this.dsp / (0.02136327583 * (14 ** ((this.sr - 14.75) / 3.9)) + this.sr / 152 + 0.022) - 1) * -0.25)) / 3.5) + 0.5).toFixed(4))
      this.plonk = Number(Number((((this.ge / (this.sr / 350 + 0.005948424455 * 3.8 ** ((this.sr - 6.1) / 4) + 0.006) - 1) + (this.app / (0.1368803292 * 1.0024 ** ((this.sr ** 5) / 2800) + this.sr / 54) - 1) + ((this.dsp / (0.02136327583 * (14 ** ((this.sr - 14.75) / 3.9)) + this.sr / 152 + 0.022) - 1) * 0.75) + (((this.pps / this.srarea) / (0.0084264 * (2.14 ** (-2 * (this.sr / 2.7 + 1.03))) - this.sr / 5750 + 0.0067) - 1) * -1)) / 2.73) + 0.5).toFixed(4))
      this.stride = Number(Number((((((this.apm / this.srarea) / ((0.069 * 1.0017 ** ((this.sr ** 5) / 4700)) + this.sr / 360) - 1) * -0.25) + ((this.pps / this.srarea) / (0.0084264 * (2.14 ** (-2 * (this.sr / 2.7 + 1.03))) - this.sr / 5750 + 0.0067) - 1) + ((this.app / (0.1368803292 * 1.0024 ** ((this.sr ** 5) / 2800) + this.sr / 54) - 1) * -2) + ((this.dsp / (0.02136327583 * (14 ** ((this.sr - 14.75) / 3.9)) + this.sr / 152 + 0.022) - 1) * -0.5)) * 0.79) + 0.5).toFixed(4))
      this.infds = Number(Number((((this.dsp / (0.02136327583 * (14 ** ((this.sr - 14.75) / 3.9)) + this.sr / 152 + 0.022) - 1) + ((this.app / (0.1368803292 * 1.0024 ** ((this.sr ** 5) / 2800) + this.sr / 54) - 1) * -0.75) + (((this.apm / this.srarea) / ((0.069 * 1.0017 ** ((this.sr ** 5) / 4700)) + this.sr / 360) - 1) * 0.5) + ((this.vsapm / (-(((this.sr - 16) / 36) ** 2) + 2.133) - 1) * 1.5) + (((this.pps / this.srarea) / (0.0084264 * (2.14 ** (-2 * (this.sr / 2.7 + 1.03))) - this.sr / 5750 + 0.0067) - 1) * 0.5)) * 0.9) + 0.5).toFixed(4))
      if (data != null) { // If we have the individual data for this player...
        // Assign all of it
        this.id = data._id
        this.rank = data.league.rank
        //this.percent_rank = data.league.percentile_rank // Pretty much just used for !avg
        this.country = data.country
        this.games = data.league.gamesplayed
        this.wins = data.league.gameswon
        this.wr = (this.wins / this.games) * 100 // TL winrate
        this.avatar = data.avatar_revision
        // this.position = pList.map(pList => pList.name).indexOf(this.name)
        // The above works but it was horrifically slow.
      } else { // Otherwise...
        // Put dummy data in.
        this.id = -1
        this.rank = null
        this.position = pList.length + 1
        this.country = null // In-game country (conv to string to prevent null error)
        this.games = -1 // TL games played
        this.wins = -1 // TL wins
        this.wr = -1 // TL winrate
      }
    }
}
  
let me = new Player(
    "t",
    66.09,
    2.07,
    135.65,
    23684.48,
    2257.86,
    66.04
)

console.log(me)